use alloc::format;
use alloc::string::String;

/// Formats a Unix timestamp using Python-like `strftime` tokens.
///
/// Supported tokens:
/// - `%Y` year with century (e.g. `2026`)
/// - `%m` month as zero-padded decimal (`01`..`12`)
/// - `%d` day of month as zero-padded decimal (`01`..`31`)
/// - `%H` hour (24-hour clock) as zero-padded decimal (`00`..`23`)
/// - `%I` hour (12-hour clock) as zero-padded decimal (`01`..`12`)
/// - `%M` minute as zero-padded decimal (`00`..`59`)
/// - `%S` second as zero-padded decimal (`00`..`59`)
/// - `%p` locale-independent `AM`/`PM`
/// - `%%` literal `%`
pub fn format_unix_timestamp(unix_timestamp: i64, pattern: &str) -> String {
    let (year, month, day, hour, minute, second) = shared::decompose_unix_timestamp(unix_timestamp);

    let mut output = String::with_capacity(pattern.len() + 16);
    let mut characters = pattern.chars();

    while let Some(character) = characters.next() {
        if character != '%' {
            output.push(character);
            continue;
        }

        match characters.next() {
            Some('Y') => output.push_str(&format!("{:04}", year)),
            Some('m') => output.push_str(&format!("{:02}", month)),
            Some('d') => output.push_str(&format!("{:02}", day)),
            Some('H') => output.push_str(&format!("{:02}", hour)),
            Some('I') => output.push_str(&format!("{:02}", hour_12(hour))),
            Some('M') => output.push_str(&format!("{:02}", minute)),
            Some('S') => output.push_str(&format!("{:02}", second)),
            Some('p') => output.push_str(if hour < 12 { "AM" } else { "PM" }),
            Some('%') => output.push('%'),
            Some(other) => {
                output.push('%');
                output.push(other);
            }
            None => output.push('%'),
        }
    }

    output
}

const fn hour_12(hour_24: u8) -> u8 {
    match hour_24 % 12 {
        0 => 12,
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_24_hour_time() {
        let timestamp = 13 * 3600 + 5 * 60;
        assert_eq!(format_unix_timestamp(timestamp, "%H:%M"), "13:05");
    }

    #[test]
    fn format_12_hour_time_with_am_pm() {
        let midnight = 0;
        let afternoon = 13 * 3600 + 5 * 60;

        assert_eq!(format_unix_timestamp(midnight, "%I:%M %p"), "12:00 AM");
        assert_eq!(format_unix_timestamp(afternoon, "%I:%M %p"), "01:05 PM");
    }

    #[test]
    fn format_date_and_time() {
        assert_eq!(
            format_unix_timestamp(0, "%Y-%m-%d %H:%M:%S"),
            "1970-01-01 00:00:00"
        );
    }

    #[test]
    fn format_negative_unix_time() {
        assert_eq!(
            format_unix_timestamp(-1, "%Y-%m-%d %H:%M:%S"),
            "1969-12-31 23:59:59"
        );
    }
}
