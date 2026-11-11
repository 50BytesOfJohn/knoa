//! Adding up what a job's AI requests used, one row per model.

use knoa_core::Usage;

use crate::job::ModelUsage;

pub fn by_model(provider: &str, requests: Vec<Usage>) -> Vec<ModelUsage> {
    let mut rows: Vec<ModelUsage> = Vec::new();
    for request in requests {
        if let Some(row) = rows.iter_mut().find(|row| row.model == request.model) {
            add(row, &request);
        } else {
            let mut row = empty_row(provider, request.model.clone());
            add(&mut row, &request);
            rows.push(row);
        }
    }
    rows
}

fn empty_row(provider: &str, model: String) -> ModelUsage {
    ModelUsage {
        provider: provider.to_owned(),
        model,
        requests: 0,
        input_tokens: 0,
        output_tokens: 0,
        cost: 0.0,
        unmetered: 0,
        unpriced: 0,
    }
}

fn add(row: &mut ModelUsage, request: &Usage) {
    row.requests += 1;
    row.input_tokens += request.input_tokens.unwrap_or_default();
    row.output_tokens += request.output_tokens.unwrap_or_default();
    row.cost += request.cost.unwrap_or_default();
    if request.input_tokens.is_none() || request.output_tokens.is_none() {
        row.unmetered += 1;
    }
    if request.cost.is_none() {
        row.unpriced += 1;
    }
}
