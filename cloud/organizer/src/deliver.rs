//! Handing a queued job's result to the API Worker over its service binding.

use worker::{Env, Headers, Method, RequestInit, console_log};

use crate::job::JobResult;

const BINDING: &str = "API";
// A service binding ignores the host; only the path reaches the API Worker.
const RESULT_URL: &str = "https://api/internal/organizer/result";

pub enum Delivery {
    Delivered,
    /// Worth trying again: the API was unreachable, overloaded or failing.
    Transient(String),
    /// Retrying won't help, say a rejected secret or a result the API refuses.
    Rejected(String),
}

pub async fn deliver(env: &Env, result: &JobResult) -> Delivery {
    let Ok(api) = env.service(BINDING) else {
        // Local runs have no API Worker: the log line is the delivery.
        console_log!("no {BINDING} binding; result for {}", result.bookmark_id);
        return Delivery::Delivered;
    };
    let init = match request(env, result) {
        Ok(init) => init,
        Err(err) => return Delivery::Rejected(err.to_string()),
    };
    match api.fetch(RESULT_URL, Some(init)).await {
        Ok(response) => match response.status_code() {
            200..=299 => Delivery::Delivered,
            status @ (429 | 500..) => Delivery::Transient(format!("the API answered {status}")),
            status => Delivery::Rejected(format!("the API answered {status}")),
        },
        Err(err) => Delivery::Transient(err.to_string()),
    }
}

fn request(env: &Env, result: &JobResult) -> worker::Result<RequestInit> {
    let headers = Headers::new();
    headers.set("content-type", "application/json")?;
    headers.set(
        "x-internal-secret",
        &env.secret("INTERNAL_SECRET")?.to_string(),
    )?;
    let body = serde_json::to_string(result)?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(body.into()));
    Ok(init)
}
