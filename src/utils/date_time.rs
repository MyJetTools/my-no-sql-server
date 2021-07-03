use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

pub fn get_utc_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn to_iso_string(timestamp: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_millis(timestamp as u64);

    let datetime = DateTime::<Utc>::from(d);

    return format!("{:?}", datetime);
}

/*
const START_ZERO: u8 = '0' as u8;

pub fn parse_iso_string(src: &[u8]) -> Option<i64> {
    let year = (src[0] - START_ZERO) as i32 * 1000
        + (src[1] - START_ZERO) as i32 * 100
        + (src[2] - START_ZERO) as i32 * 10
        + (src[3] - START_ZERO) as i32;

    let month = (src[5] - START_ZERO) as u32 * 10 + (src[6] - START_ZERO) as u32;

    let day = (src[8] - START_ZERO) as u32 * 10 + (src[9] - START_ZERO) as u32;

    let hour = (src[11] - START_ZERO) as u32 * 10 + (src[12] - START_ZERO) as u32;

    let min = (src[14] - START_ZERO) as u32 * 10 + (src[15] - START_ZERO) as u32;

    let sec = (src[17] - START_ZERO) as u32 * 10 + (src[18] - START_ZERO) as u32;

    let msec = (src[20] - START_ZERO) as u32 * 100
        + (src[21] - START_ZERO) as u32 * 10
        + (src[22] - START_ZERO) as u32;

    let date_time = NaiveDate::from_ymd(year, month, day).and_hms_milli(hour, min, sec, msec);

    Some(date_time.timestamp_millis())
}


#[cfg(test)]
mod tests {
    #[test]
    pub fn parse_iso_string() {
        let src = "2021-04-25T17:30:43.605Z";

        let result = super::parse_iso_string(src.as_bytes());
        assert_eq!(1619371843605, result.unwrap());
    }

    #[test]
    pub fn test_utc_now() {
        let result = super::get_utc_now();
        println!("{}", result);
        println!("{}", super::to_iso_string(result));
    }
}
 */
