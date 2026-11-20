//! Counts and amounts: the language's digits and decimal separator, and the region's
//! grouping, as in "1,234.5", "1 234,5" or "1.234,5".

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use knoa_core::Language;
use objc2::rc::Retained;
use objc2_foundation::{NSNumber, NSNumberFormatter, NSNumberFormatterStyle, NSString};

use super::locale;
use crate::i18n;

/// A count or a measure: "12,345", "2.5".
pub fn number(value: f64) -> String {
    write(
        value,
        Style::Decimal {
            fraction_digits: 0..=2,
        },
    )
}

/// An amount in US dollars, which AI providers bill in, with exactly `fraction_digits`
/// after the separator: "$0.12", "0,12 $US".
pub fn usd(amount: f64, fraction_digits: u8) -> String {
    write(amount, Style::Usd { fraction_digits })
}

/// A count rounded to thousands or millions with one decimal: "950", "1.2K", "3,4M".
pub fn compact(count: u64) -> String {
    let (scaled, suffix) = match count {
        0..1_000 => return number(count as f64),
        1_000..1_000_000 => (count as f64 / 1e3, "K"),
        _ => (count as f64 / 1e6, "M"),
    };
    let digits = write(
        scaled,
        Style::Decimal {
            fraction_digits: 1..=1,
        },
    );
    format!("{digits}{suffix}")
}

/// What a formatter writes, and so which cached formatter to use.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Style {
    Decimal {
        fraction_digits: std::ops::RangeInclusive<u8>,
    },
    Usd {
        fraction_digits: u8,
    },
}

type Key = (Language, Style);

static FORMATTERS: LazyLock<Mutex<HashMap<Key, Retained<NSNumberFormatter>>>> =
    LazyLock::new(Mutex::default);

fn write(value: f64, style: Style) -> String {
    let formatter = formatter(style);
    formatter
        .stringFromNumber(&NSNumber::new_f64(value))
        .map_or_else(|| value.to_string(), |text| text.to_string())
}

fn formatter(style: Style) -> Retained<NSNumberFormatter> {
    let language = i18n::language();
    let mut formatters = FORMATTERS.lock().expect("lock poisoned");
    let formatter = formatters
        .entry((language, style.clone()))
        .or_insert_with(|| {
            let formatter = NSNumberFormatter::new();
            formatter.setLocale(Some(&locale(language)));
            let (min, max) = match style {
                Style::Decimal { fraction_digits } => {
                    formatter.setNumberStyle(NSNumberFormatterStyle::DecimalStyle);
                    (*fraction_digits.start(), *fraction_digits.end())
                }
                Style::Usd { fraction_digits } => {
                    formatter.setNumberStyle(NSNumberFormatterStyle::CurrencyStyle);
                    formatter.setCurrencyCode(Some(&NSString::from_str("USD")));
                    (fraction_digits, fraction_digits)
                }
            };
            formatter.setMinimumFractionDigits(usize::from(min));
            formatter.setMaximumFractionDigits(usize::from(max));
            formatter
        });
    formatter.clone()
}
