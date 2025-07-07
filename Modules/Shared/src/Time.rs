pub fn Unix_to_human_time(Unix_timestamp: i64) -> (u16, u8, u8, u8, u8, u8) {
    // Constants for calculations
    const SECONDS_IN_MINUTE: i64 = 60;
    const SECONDS_IN_HOUR: i64 = 60 * SECONDS_IN_MINUTE;
    const SECONDS_IN_DAY: i64 = 24 * SECONDS_IN_HOUR;
    const DAYS_IN_YEAR: i64 = 365;
    const DAYS_IN_LEAP_YEAR: i64 = 366;

    // Start from 1970
    let mut Year = 1970;
    let mut Days_since_epoch = Unix_timestamp / SECONDS_IN_DAY;
    let mut Remaining_seconds = Unix_timestamp % SECONDS_IN_DAY;

    if Remaining_seconds < 0 {
        // Handle negative Unix timestamps
        Days_since_epoch -= 1;
        Remaining_seconds += SECONDS_IN_DAY;
    }

    // Determine the current year
    while Days_since_epoch
        >= if Is_leap_year(Year) {
            DAYS_IN_LEAP_YEAR
        } else {
            DAYS_IN_YEAR
        }
    {
        Days_since_epoch -= if Is_leap_year(Year) {
            DAYS_IN_LEAP_YEAR
        } else {
            DAYS_IN_YEAR
        };
        Year += 1;
    }

    // Determine the current month and day
    let mut Month = 0;
    while Days_since_epoch >= Days_in_month(Year, Month) {
        Days_since_epoch -= Days_in_month(Year, Month);
        Month += 1;
    }

    // Remaining days are the day of the month
    let Day = Days_since_epoch + 1;

    // Calculate hour, minute, and second from remaining seconds
    let Hour = Remaining_seconds / SECONDS_IN_HOUR;
    Remaining_seconds %= SECONDS_IN_HOUR;
    let Minute = Remaining_seconds / SECONDS_IN_MINUTE;
    let Second = Remaining_seconds % SECONDS_IN_MINUTE;

    (
        Year as u16,
        Month as u8 + 1,
        Day as u8,
        Hour as u8,
        Minute as u8,
        Second as u8,
    )
}

pub fn Is_leap_year(Year: i64) -> bool {
    (Year % 4 == 0 && Year % 100 != 0) || (Year % 400 == 0)
}

pub fn Days_in_month(Year: i64, Month: usize) -> i64 {
    // Number of days in each month (non-leap year)
    const DAYS_IN_MONTH: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    if Month == 1 && Is_leap_year(Year) {
        // February in a leap year
        29
    } else {
        DAYS_IN_MONTH[Month]
    }
}
