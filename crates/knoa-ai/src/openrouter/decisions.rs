//! The Decisions API (alpha): yes/no questions answered with probabilities.

use serde::Deserialize;
use serde_json::{Map, Value, json};

use super::OpenRouter;
use crate::{Classifier, Error, Question};

/// Decision models cap the questions per request (64 for some); stay under the lowest.
const QUESTIONS_PER_REQUEST: usize = 60;
const SENTINEL_KEY: &str = "zz_sentinel";

#[derive(Deserialize)]
struct Decisions {
    answers: Map<String, Value>,
}

impl Classifier for OpenRouter {
    async fn classify(&self, state: &Value, questions: &[Question]) -> Result<Vec<f32>, Error> {
        let Some(model) = &self.classifier_model else {
            return Err(Error::BadAnswer("no classifier model configured".into()));
        };
        let mut probabilities = Vec::with_capacity(questions.len());
        for chunk in questions.chunks(QUESTIONS_PER_REQUEST) {
            let decisions: Decisions = self
                .post("/api/alpha/decisions", &body(model, state, chunk))
                .await?;
            for question in chunk {
                let p = decisions
                    .answers
                    .get(&question.key)
                    .and_then(|a| a.get("noul"))
                    .and_then(Value::as_f64);
                probabilities
                    .push(p.ok_or_else(|| {
                        Error::BadAnswer(format!("no answer for {}", question.key))
                    })? as f32);
            }
        }
        Ok(probabilities)
    }
}

fn body(model: &str, state: &Value, questions: &[Question]) -> Value {
    let mut map = Map::new();
    for q in questions {
        map.insert(q.key.clone(), json!({ "type": "noul", "instructions": q.instructions, "criteria": { "true": q.yes, "false": q.no } }));
    }
    // Some decision models inflate the last question's probability. A throwaway question
    // in last place absorbs that bias; its key sorts after every real one.
    map.insert(
        SENTINEL_KEY.into(),
        json!({ "type": "noul", "instructions": "Is this bookmark written entirely in Klingon?",
                "criteria": { "true": "The content is in Klingon.", "false": "It is not in Klingon." } }),
    );
    json!({ "model": model, "state": state, "questions": map })
}
