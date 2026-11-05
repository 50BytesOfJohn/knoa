//! The Decisions API (alpha): yes/no and choice questions answered with probabilities.

use serde::Deserialize;
use serde_json::{Map, Value, json};

use super::OpenRouter;
use crate::{Choice, Classifier, Error, Question};

/// Decision models cap the questions per request (64 for some); stay under the lowest.
const QUESTIONS_PER_REQUEST: usize = 60;
const SENTINEL_KEY: &str = "zz_sentinel";
const CHOICE_KEY: &str = "choice";

#[derive(Deserialize)]
struct Decisions {
    answers: Map<String, Value>,
}

impl Classifier for OpenRouter {
    async fn classify(&self, state: &Value, questions: &[Question]) -> Result<Vec<f32>, Error> {
        let model = self.decision_model()?;
        let mut probabilities = Vec::with_capacity(questions.len());
        for chunk in questions.chunks(QUESTIONS_PER_REQUEST) {
            let decisions: Decisions = self
                .post("/api/alpha/decisions", &yes_no_body(model, state, chunk))
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

    async fn choose(&self, state: &Value, choice: &Choice) -> Result<Vec<f32>, Error> {
        let model = self.decision_model()?;
        let decisions: Decisions = self
            .post("/api/alpha/decisions", &choice_body(model, state, choice))
            .await?;
        let probabilities = decisions
            .answers
            .get(CHOICE_KEY)
            .and_then(|a| a.get("probabilities"))
            .ok_or_else(|| Error::BadAnswer("no answer for the choice".into()))?;
        // An option the model left out had no chance.
        Ok(choice
            .options
            .iter()
            .map(|(key, _)| {
                probabilities
                    .get(key)
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0) as f32
            })
            .collect())
    }
}

impl OpenRouter {
    fn decision_model(&self) -> Result<&str, Error> {
        self.classifier_model
            .as_deref()
            .ok_or_else(|| Error::BadAnswer("no classifier model configured".into()))
    }
}

fn yes_no_body(model: &str, state: &Value, questions: &[Question]) -> Value {
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

/// One question per request, so there is no last question to bias and no sentinel.
fn choice_body(model: &str, state: &Value, choice: &Choice) -> Value {
    let criteria: Map<String, Value> = choice
        .options
        .iter()
        .map(|(key, when)| (key.clone(), Value::from(when.as_str())))
        .collect();
    json!({
        "model": model,
        "state": state,
        "questions": { CHOICE_KEY: { "type": "choice", "instructions": choice.instructions, "criteria": criteria } },
    })
}
