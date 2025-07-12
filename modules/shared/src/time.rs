pub fn unix_to_human_time(unix_timestamp: i64) -> (u16, u8, u8, u8, u8, u8) {
    // Constants for calculations
    const SECONDS_IN_MINUTE: i64 = 60;
    const SECONDS_IN_HOUR: i64 = 60 * SECONDS_IN_MINUTE;
    const SECONDS_IN_DAY: i64 = 24 * SECONDS_IN_HOUR;
    const DAYS_IN_YEAR: i64 = 365;
    const DAYS_IN_LEAP_YEAR: i64 = 366;

    // Start from 1970
    let mut year = 1970;
    let mut days_since_epoch = unix_timestamp / SECONDS_IN_DAY;
    let mut remaining_seconds = unix_timestamp % SECONDS_IN_DAY;

    if remaining_seconds < 0 {
        // Handle negative Unix timestamps
        days_since_epoch -= 1;
        remaining_seconds += SECONDS_IN_DAY;
    }

    // Determine the current year
    while days_since_epoch
        >= if is_leap_year(year) {
            DAYS_IN_LEAP_YEAR
        } else {
            DAYS_IN_YEAR
        }
    {
        days_since_epoch -= if is_leap_year(year) {
            DAYS_IN_LEAP_YEAR
        } else {
            DAYS_IN_YEAR
        };
        year += 1;
    }

    // Determine the current month and day
    let mut month = 0;
    while days_since_epoch >= days_in_month(year, month) {
        days_since_epoch -= days_in_month(year, month);
        month += 1;
    }

    // Remaining days are the day of the month
    let day = days_since_epoch + 1;

    // Calculate hour, minute, and second from remaining seconds
    let hour = remaining_seconds / SECONDS_IN_HOUR;
    remaining_seconds %= SECONDS_IN_HOUR;
    let minute = remaining_seconds / SECONDS_IN_MINUTE;
    let second = remaining_seconds % SECONDS_IN_MINUTE;

    (
        year as u16,
        month as u8 + 1,
        day as u8,
        hour as u8,
        minute as u8,
        second as u8,
    )
}

pub fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: i64, month: usize) -> i64 {
    // Number of days in each month (non-leap year)
    const DAYS_IN_MONTH: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    if month == 1 && is_leap_year(year) {
        // February in a leap year
        29
    } else {
        DAYS_IN_MONTH[month]
    }
}
