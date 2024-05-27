use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use std::convert::TryInto;
use std::error::Error;

const U16_MICROSECOND_STEP: f64 = 15.2587890625;

pub fn time_to_ble_data(current_time: DateTime<Utc>) -> Vec<u8> {
    let year_bytes = (current_time.year() as u16).to_le_bytes();
    let month = current_time.month() as u8;
    let day = current_time.day() as u8;
    let hour = current_time.hour() as u8;
    let minute = current_time.minute() as u8;
    let second = current_time.second() as u8;
    let microsecond = current_time.timestamp_subsec_micros();
    let fractions_of_second =
        (((microsecond as f64) / U16_MICROSECOND_STEP).round() as u16).to_le_bytes();
    let day_of_week = current_time.weekday().num_days_from_monday() as u8 + 1;

    vec![
        year_bytes[0],
        year_bytes[1],
        month,
        day,
        hour,
        minute,
        second,
        day_of_week,
        fractions_of_second[0],
        fractions_of_second[1],
        0,
    ]
}

#[allow(deprecated)]
pub fn ble_data_to_time(data: &[u8]) -> Result<DateTime<Utc>, Box<dyn Error>> {
    if data.len() == 11 {
        let year = u16::from_le_bytes(data[0..2].try_into()?) as i32;
        let month = data[2] as u32;
        let day = data[3] as u32;
        let hour = data[4] as u32;
        let minute = data[5] as u32;
        let second = data[6] as u32;
        let fractions = u16::from_le_bytes(data[8..10].try_into()?);

        let fractions_in_us = (fractions as f64 * U16_MICROSECOND_STEP).round() as u32;

        let date_time =
            Utc.ymd(year, month, day)
                .and_hms_micro(hour, minute, second, fractions_in_us);

        Ok(date_time)
    } else {
        Err("Invalid BLE time data format".into())
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_time_to_ble_data() {
        let time = Utc.ymd(2023, 4, 1).and_hms_micro(12, 34, 56, 789001); // 789001 us, because 789001 / 15.2587890625 rounds nicely t0 51708 (0xC9FC)
        let expected_data = vec![0xE7, 0x07, 4, 1, 12, 34, 56, 5, 0xFC, 0xC9, 0];
        let ble_data = time_to_ble_data(time);
        assert_eq!(ble_data.len(), 11);
        assert_eq!(ble_data, expected_data);
    }

    #[test]
    fn test_ble_data_to_time() {
        let data = vec![0xE7, 0x07, 4, 1, 12, 34, 56, 5, 0xFC, 0xC9, 0];
        let expected_time = Utc.ymd(2023, 4, 1).and_hms_micro(12, 34, 56, 789001);
        match ble_data_to_time(&data) {
            Ok(time) => assert_eq!(time, expected_time),
            Err(e) => panic!("Failed to convert BLE data to time: {}", e),
        }
    }

    #[test]
    fn test_time_to_ble_data_to_time() {
        let time = Utc.ymd(2023, 4, 1).and_hms_micro(12, 34, 56, 789001);
        let ble_data = time_to_ble_data(time);
        let converted_time = ble_data_to_time(&ble_data).unwrap();
        assert_eq!(time, converted_time);
    }

    #[test]
    fn test_ble_data_to_time_to_ble_data() {
        let data = vec![0xE7, 0x07, 4, 1, 12, 34, 56, 5, 0x34, 0x12, 0];
        let time = ble_data_to_time(&data).unwrap();
        let ble_data = time_to_ble_data(time);
        assert_eq!(data, ble_data);
    }

    #[test]
    fn test_ble_data_to_time_invalid_data() {
        let data = vec![0xE7, 0x07, 4, 1, 12, 34, 56, 1, 0x34, 0x12];
        match ble_data_to_time(&data) {
            Ok(_) => panic!("Expected error for invalid BLE data format"),
            Err(_) => {}
        }
    }
}
