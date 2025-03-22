use chrono::{DateTime, FixedOffset};

pub fn datetime_to_string(datetime: DateTime<FixedOffset>) -> String {
    let offset = FixedOffset::east_opt(7 * 60 * 60).unwrap(); // +0700
    datetime
        .with_timezone(&offset)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

pub fn datetime_to_string_opt(datetime: Option<DateTime<FixedOffset>>) -> Option<String> {
    datetime?;
    let offset = FixedOffset::east_opt(7 * 60 * 60).unwrap(); // +0700
    Some(
        datetime
            .unwrap()
            .with_timezone(&offset)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    )
}
