//! The cloud organizer: Knoa's fetch and AI pipeline in a Cloudflare Worker.
//!
//! Jobs arrive on a queue and their results go to the API Worker. `POST /organize` runs
//! one job and answers with its result, for testing.

mod deliver;
mod job;
mod organize;
mod usage;

use futures::future::join_all;
use serde_json::{Value, json};
use worker::{
    Context, Env, Message, MessageBatch, MessageExt as _, Method, QueueRetryOptionsBuilder,
    Request, Response, console_error, console_log, event,
};

use crate::{
    deliver::Delivery,
    job::{ApiKeySource, Job},
    organize::Outcome,
};

/// Jobs the queue gave up on land in a queue named like this, in every environment, so
/// their bookmarks still get a result.
const FAILED_QUEUE_MARK: &str = "-dlq";
/// How long a result waits when the API Worker couldn't take it.
const DELIVERY_RETRY_SECONDS: u32 = 60;

#[event(fetch)]
async fn fetch(mut req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    if req.method() != Method::Post || req.path() != "/organize" {
        return Response::error("not found", 404);
    }
    if !is_internal(&req, &env)? {
        return Response::error("forbidden", 403);
    }
    let Ok(job) = req.json::<Job>().await else {
        return Response::error("not a valid job", 400);
    };
    match organize::run(&job, api_key(&env, job.api_key_source)?).await {
        Outcome::Finished(result) => Response::from_json(&result),
        Outcome::Retry { error, after } => Ok(Response::from_json(&json!({
            "error": error,
            "retry_after": after.as_secs(),
        }))?
        .with_status(503)),
    }
}

#[event(queue)]
async fn queue(batch: MessageBatch<Value>, env: Env, _ctx: Context) -> worker::Result<()> {
    let gave_up = batch.queue().contains(FAILED_QUEUE_MARK);
    let messages = batch.messages()?;
    join_all(
        messages
            .iter()
            .map(|message| handle(message, gave_up, &env)),
    )
    .await;
    Ok(())
}

async fn handle(message: &Message<Value>, gave_up: bool, env: &Env) {
    let job = match serde_json::from_value::<Job>(message.body().clone()) {
        Ok(job) => job,
        Err(err) => {
            // Nothing to deliver without a bookmark id, and retrying won't parse it either.
            console_error!("dropping message {}: {err}", message.id());
            message.ack();
            return;
        }
    };
    let result = if gave_up {
        let error = "organizing kept failing; gave up after several tries";
        organize::failure(&job, error.into())
    } else {
        let api_key = match api_key(env, job.api_key_source) {
            Ok(key) => key,
            Err(err) => {
                console_error!("no API key: {err}");
                retry(message, DELIVERY_RETRY_SECONDS);
                return;
            }
        };
        match organize::run(&job, api_key).await {
            Outcome::Finished(result) => result,
            Outcome::Retry { error, after } => {
                console_log!("retrying {} for {}: {error}", job.bookmark_id, job.user_id);
                retry(message, after.as_secs() as u32);
                return;
            }
        }
    };
    match deliver::deliver(env, &result).await {
        Delivery::Delivered => message.ack(),
        Delivery::Transient(err) => {
            console_log!("result for {} not delivered: {err}", job.bookmark_id);
            retry(message, DELIVERY_RETRY_SECONDS);
        }
        Delivery::Rejected(err) => {
            console_error!("the API refused the result for {}: {err}", job.bookmark_id);
            message.ack();
        }
    }
}

fn retry(message: &Message<Value>, delay_seconds: u32) {
    let options = QueueRetryOptionsBuilder::new()
        .with_delay_seconds(delay_seconds)
        .build();
    message.retry_with_options(&options);
}

fn api_key(env: &Env, source: ApiKeySource) -> worker::Result<String> {
    match source {
        ApiKeySource::Ours => Ok(env.secret("OPENROUTER_API_KEY")?.to_string()),
    }
}

/// `POST /organize` spends our AI credits, so only callers holding the internal secret
/// may use it.
fn is_internal(req: &Request, env: &Env) -> worker::Result<bool> {
    let expected = env.secret("INTERNAL_SECRET")?.to_string();
    let given = req.headers().get("x-internal-secret")?.unwrap_or_default();
    Ok(constant_time_eq(given.as_bytes(), expected.as_bytes()))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |diff, (x, y)| diff | (x ^ y)) == 0
}
