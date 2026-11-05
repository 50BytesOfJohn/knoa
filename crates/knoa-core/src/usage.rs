//! What AI requests used and cost, as far as the provider reported it.

/// One AI request. Providers report what they can, so each measure may be missing.
#[derive(Debug, Clone, PartialEq)]
pub struct Usage {
    /// The model that answered.
    pub model: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    /// In US dollars.
    pub cost: Option<f64>,
}

/// Requests added up. Requests the provider didn't report tokens or a cost for are
/// counted, so a total can say it is incomplete instead of looking lower than it was.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UsageTotals {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
    /// Requests without token counts.
    pub unmetered: u64,
    /// Requests without a cost.
    pub unpriced: u64,
}

impl UsageTotals {
    pub fn add(&mut self, other: &UsageTotals) {
        self.requests += other.requests;
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cost += other.cost;
        self.unmetered += other.unmetered;
        self.unpriced += other.unpriced;
    }
}

/// One model's totals over some days.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelUsage {
    /// The provider's settings id, like `openrouter`.
    pub provider: String,
    pub model: String,
    pub totals: UsageTotals,
}
