//! Typed kind fields and coercion of loosely typed values (from the AI or page metadata).

use serde::Deserialize;
use serde_json::{Map, Value};

/// A bookmark's field values, keyed by field name. Stored as one JSON object so that
/// filters can use `json_extract`.
pub type Fields = Map<String, Value>;

#[derive(Debug, Deserialize)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: FieldType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Integer,
    Number,
    /// `YYYY-MM-DD`, or a shorter prefix (`YYYY-MM`, `YYYY`) when that's all that is known.
    Date,
    /// Whole minutes.
    Duration,
    List,
}

impl FieldDef {
    pub fn label(&self) -> String {
        let mut label = self.name.replace('_', " ");
        if let Some(first) = label.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        label
    }

    /// Turns a value of roughly the right shape into the field's type, or `None` if it
    /// can't be read as one. Models return `"148"`, `"2h 28m"` or `"PT2H28M"` for a runtime.
    pub fn coerce(&self, value: &Value) -> Option<Value> {
        let text = match value {
            Value::Null => return None,
            Value::String(s) if s.trim().is_empty() || is_unknown(s) => return None,
            Value::String(s) => Some(s.trim()),
            _ => None,
        };
        match self.ty {
            FieldType::Text => text
                .map(|t| Value::from(t.to_owned()))
                .or_else(|| scalar(value)),
            FieldType::Integer => as_number(value, text).map(|n| Value::from(n.round() as i64)),
            FieldType::Number => as_number(value, text)
                .and_then(|n| serde_json::Number::from_f64(n).map(Value::Number)),
            FieldType::Duration => match text {
                Some(t) => parse_minutes(t).map(Value::from),
                None => as_number(value, None).map(|n| Value::from(n.round() as i64)),
            },
            FieldType::Date => text.and_then(parse_date).map(Value::from),
            FieldType::List => as_list(value),
        }
    }

    /// The value as a short display string.
    pub fn display(&self, value: &Value) -> String {
        match (self.ty, value) {
            (FieldType::Duration, Value::Number(n)) => {
                n.as_i64().map_or_else(String::new, format_duration)
            }
            (_, Value::Array(items)) => items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            (_, Value::String(s)) => s.clone(),
            (_, other) => other.to_string(),
        }
    }
}

/// `148` → `2h 28m`, `45` → `45 min`.
pub fn format_duration(minutes: i64) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m} min"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}

/// Reads `90`, `90 min`, `1h 30m`, `1:30:00` and ISO 8601 `PT1H30M` as minutes.
pub(crate) fn parse_minutes(text: &str) -> Option<i64> {
    let lower = text.to_ascii_lowercase();
    if let Ok(n) = lower
        .trim_end_matches("minutes")
        .trim_end_matches("min")
        .trim()
        .parse::<f64>()
    {
        return Some(n.round() as i64);
    }
    if lower.contains(':') {
        let parts: Vec<i64> = lower
            .split(':')
            .map(|p| p.trim().parse().ok())
            .collect::<Option<_>>()?;
        return match parts.as_slice() {
            [h, m, _s] => Some(h * 60 + m),
            [m, s] => Some(m + i64::from(*s >= 30)),
            _ => None,
        };
    }
    let body = lower.strip_prefix("pt").unwrap_or(&lower);
    let mut total = 0.0;
    let mut number = String::new();
    let mut seen_unit = false;
    for ch in body.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
        } else if matches!(ch, 'h' | 'm' | 's') && !number.is_empty() {
            let n: f64 = number.parse().ok()?;
            total += match ch {
                'h' => n * 60.0,
                'm' => n,
                _ => n / 60.0,
            };
            number.clear();
            seen_unit = true;
        }
    }
    (seen_unit && number.is_empty()).then(|| total.round() as i64)
}

fn parse_date(text: &str) -> Option<String> {
    let date = text.get(..10.min(text.len()))?;
    let valid = date.split('-').enumerate().all(|(i, part)| {
        let len = if i == 0 { 4 } else { 2 };
        part.len() == len && part.bytes().all(|b| b.is_ascii_digit())
    });
    (valid && date.len() >= 4).then(|| date.to_owned())
}

fn as_number(value: &Value, text: Option<&str>) -> Option<f64> {
    match (value, text) {
        (Value::Number(n), _) => n.as_f64(),
        (_, Some(t)) => {
            let cleaned: String = t
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let n: f64 = cleaned.parse().ok()?;
            let lower = t.to_ascii_lowercase();
            Some(if lower.ends_with('k') { n * 1000.0 } else { n })
        }
        _ => None,
    }
}

fn as_list(value: &Value) -> Option<Value> {
    let items: Vec<Value> = match value {
        Value::Array(items) => items
            .iter()
            .filter_map(scalar_string)
            .map(Value::from)
            .collect(),
        Value::String(s) => s
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(Value::from)
            .collect(),
        _ => return None,
    };
    (!items.is_empty()).then_some(Value::Array(items))
}

fn scalar(value: &Value) -> Option<Value> {
    scalar_string(value).map(Value::from)
}

fn scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) if !s.trim().is_empty() && !is_unknown(s) => Some(s.trim().to_owned()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn is_unknown(text: &str) -> bool {
    matches!(
        text.trim().to_ascii_lowercase().as_str(),
        "unknown" | "n/a" | "none" | "null" | "-"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn def(ty: FieldType) -> FieldDef {
        FieldDef {
            name: "f".into(),
            ty,
            description: String::new(),
        }
    }

    #[test]
    fn durations_in_common_formats_become_minutes() {
        let duration = def(FieldType::Duration);
        for (input, minutes) in [
            (json!("PT2H28M"), 148),
            (json!("2h 28m"), 148),
            (json!("148 min"), 148),
            (json!(148), 148),
            (json!("1:02:40"), 62),
        ] {
            assert_eq!(duration.coerce(&input), Some(json!(minutes)), "{input}");
        }
        assert_eq!(duration.coerce(&json!("unknown")), None);
    }

    #[test]
    fn integers_accept_thousands_shorthand() {
        assert_eq!(
            def(FieldType::Integer).coerce(&json!("12.3k")),
            Some(json!(12300))
        );
    }
}
