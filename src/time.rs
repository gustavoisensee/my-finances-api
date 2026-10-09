use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use serde::Serializer;

/// Columns are Prisma `TIMESTAMP(3)` values stored in UTC. Serialized the same
/// way Prisma/`JSON.stringify` does: `2025-12-07T21:37:31.000Z`.
pub fn serialize<S: Serializer>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format(value))
}

pub fn format(value: &NaiveDateTime) -> String {
    value.and_utc().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Parses a client-supplied ISO-8601 date-time into a UTC timestamp with
/// millisecond precision. Offset-less values are treated as UTC.
pub fn parse(value: &str) -> Option<NaiveDateTime> {
    let value = value.trim();
    let parsed = DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc).naive_utc())
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").ok())?;
    Some(truncate_to_millis(parsed))
}

pub fn now() -> NaiveDateTime {
    truncate_to_millis(Utc::now().naive_utc())
}

fn truncate_to_millis(value: NaiveDateTime) -> NaiveDateTime {
    let millis = value.and_utc().timestamp_millis();
    DateTime::from_timestamp_millis(millis)
        .map(|dt| dt.naive_utc())
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_prisma() {
        let dt = parse("2025-12-07T21:37:31Z").unwrap();
        assert_eq!(format(&dt), "2025-12-07T21:37:31.000Z");
    }

    #[test]
    fn parses_offsets_and_truncates() {
        let dt = parse("2025-01-01T02:00:00.123456+02:00").unwrap();
        assert_eq!(format(&dt), "2025-01-01T00:00:00.123Z");
        let naive = parse("2025-01-15T10:30:00").unwrap();
        assert_eq!(format(&naive), "2025-01-15T10:30:00.000Z");
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(parse("2025-01-01").is_none());
        assert!(parse("not a date").is_none());
        assert!(parse("").is_none());
    }
}
