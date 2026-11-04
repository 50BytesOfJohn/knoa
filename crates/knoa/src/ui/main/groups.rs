//! Date groups for the library: Today, Yesterday, This week, Last week, then months of
//! this year, then years.

use jiff::{Timestamp, Zoned, civil::Date, tz::TimeZone};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    Month(i8),
    Year(i16),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub period: Option<Period>,
    pub label: String,
    /// The range of bookmarks in the (newest-first) result list.
    pub start: usize,
    pub len: usize,
}

/// Splits newest-first timestamps into date groups.
pub fn by_date(added: &[i64], now: &Zoned) -> Vec<Group> {
    let today = now.date();
    let mut groups: Vec<Group> = Vec::new();
    for (index, &seconds) in added.iter().enumerate() {
        let period = period_of(local_date(seconds, now.time_zone()), today);
        match groups.last_mut() {
            Some(group) if group.period == Some(period) => group.len += 1,
            _ => groups.push(Group {
                period: Some(period),
                label: label(period),
                start: index,
                len: 1,
            }),
        }
    }
    groups
}

/// One group holding everything, for relevance-ranked search results.
pub fn single(label: &str, len: usize) -> Vec<Group> {
    if len == 0 {
        Vec::new()
    } else {
        vec![Group {
            period: None,
            label: label.into(),
            start: 0,
            len,
        }]
    }
}

fn local_date(seconds: i64, tz: &TimeZone) -> Date {
    Timestamp::from_second(seconds)
        .unwrap_or(Timestamp::UNIX_EPOCH)
        .to_zoned(tz.clone())
        .date()
}

fn period_of(date: Date, today: Date) -> Period {
    let days_ago = (today - date).get_days();
    let weekday = i32::from(today.weekday().to_monday_zero_offset());
    match days_ago {
        ..=0 => Period::Today,
        1 => Period::Yesterday,
        d if d <= weekday => Period::ThisWeek,
        d if d <= weekday + 7 => Period::LastWeek,
        _ if date.year() == today.year() => Period::Month(date.month()),
        _ => Period::Year(date.year()),
    }
}

fn label(period: Period) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    match period {
        Period::Today => "Today".into(),
        Period::Yesterday => "Yesterday".into(),
        Period::ThisWeek => "This week".into(),
        Period::LastWeek => "Last week".into(),
        Period::Month(m) => MONTHS[usize::try_from(m - 1).unwrap_or(0)].into(),
        Period::Year(y) => y.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_follow_calendar_periods() {
        // Wednesday 2026-11-04, noon UTC.
        let now: Zoned = "2026-11-04T12:00:00+00:00[UTC]".parse().unwrap();
        let day = 86_400;
        let t = now.timestamp().as_second();
        let added = [
            t,
            t - day,
            t - 2 * day,
            t - 5 * day,
            t - 40 * day,
            t - 400 * day,
        ];
        let labels: Vec<_> = by_date(&added, &now)
            .into_iter()
            .map(|g| (g.label, g.len))
            .collect();
        assert_eq!(
            labels,
            [
                ("Today", 1),
                ("Yesterday", 1),
                ("This week", 1),
                ("Last week", 1),
                ("September", 1),
                ("2025", 1)
            ]
            .map(|(l, n)| (l.to_owned(), n))
        );
    }
}
