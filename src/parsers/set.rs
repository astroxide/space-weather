use alloc::string::String;
use alloc::vec::Vec;
use core::str;

use crate::{Date, SpaceWeatherError, SpaceWeatherRecord};

fn skip_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

fn parse_int<T: str::FromStr>(slice: &str, row: usize, field: &str) -> Result<T, SpaceWeatherError> {
    slice.trim().parse().map_err(|_| SpaceWeatherError::ParseError {
        row,
        message: String::from(field),
    })
}

fn parse_float(slice: &str, row: usize, field: &str) -> Result<f64, SpaceWeatherError> {
    slice.trim().parse().map_err(|_| SpaceWeatherError::ParseError {
        row,
        message: String::from(field),
    })
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

const DAYS_IN_MONTH: [u16; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn doy_to_date(year: i32, doy: u16, row: usize) -> Result<Date, SpaceWeatherError> {
    let leap = is_leap_year(year);
    let mut remaining = doy;
    for (i, &days) in DAYS_IN_MONTH.iter().enumerate() {
        let d = if i == 1 && leap { days + 1 } else { days };
        if remaining <= d {
            return Ok(Date {
                year,
                month: (i + 1) as u8,
                day: remaining as u8,
            });
        }
        remaining -= d;
    }
    Err(SpaceWeatherError::ParseError {
        row,
        message: String::from("invalid day-of-year"),
    })
}

fn sl(line: &[u8], start: usize, end: usize) -> &str {
    unsafe { str::from_utf8_unchecked(&line[start..end]) }
}

// SOLFSMY format: I6 I4 F12.1 8×F6.1 A6  (total 76 chars)
// Fields: YYYY  DDD  JulDay  F10  F81c  S10  S81c  M10  M81c  Y10  Y81c  Ssrc
const SOLFSMY_MIN_LEN: usize = 64; // through Y10.7
const SOLFSMY_YYYY: (usize, usize) = (0, 6);
const SOLFSMY_DDD: (usize, usize) = (6, 10);
const SOLFSMY_F107: (usize, usize) = (22, 28);
const SOLFSMY_F81C: (usize, usize) = (28, 34);
const SOLFSMY_S107: (usize, usize) = (34, 40);
const SOLFSMY_M107: (usize, usize) = (46, 52);
const SOLFSMY_Y107: (usize, usize) = (58, 64);

pub fn parse_solfsmy(input: &[u8]) -> Result<Vec<SpaceWeatherRecord>, SpaceWeatherError> {
    let text = str::from_utf8(input).map_err(|_| SpaceWeatherError::ParseError {
        row: 0,
        message: String::from("invalid utf-8"),
    })?;

    let mut records = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if skip_line(line) {
            continue;
        }
        let row = i + 1;
        let b = line.as_bytes();
        if b.len() < SOLFSMY_MIN_LEN {
            return Err(SpaceWeatherError::ParseError {
                row,
                message: String::from("line too short"),
            });
        }

        let year: i32 = parse_int(sl(b, SOLFSMY_YYYY.0, SOLFSMY_YYYY.1), row, "year")?;
        let doy: u16 = parse_int(sl(b, SOLFSMY_DDD.0, SOLFSMY_DDD.1), row, "doy")?;
        let date = doy_to_date(year, doy, row)?;

        let f10_7 = parse_float(sl(b, SOLFSMY_F107.0, SOLFSMY_F107.1), row, "F10.7")?;
        let f10_7a = parse_float(sl(b, SOLFSMY_F81C.0, SOLFSMY_F81C.1), row, "F10.7a")?;
        let s10_7 = parse_float(sl(b, SOLFSMY_S107.0, SOLFSMY_S107.1), row, "S10.7")?;
        let m10_7 = parse_float(sl(b, SOLFSMY_M107.0, SOLFSMY_M107.1), row, "M10.7")?;
        let y10_7 = parse_float(sl(b, SOLFSMY_Y107.0, SOLFSMY_Y107.1), row, "Y10.7")?;

        records.push(SpaceWeatherRecord {
            date,
            f10_7: Some(f10_7),
            f10_7a: Some(f10_7a),
            s10_7: Some(s10_7),
            m10_7: Some(m10_7),
            y10_7: Some(y10_7),
            ap_daily: None,
            ap_3hr: None,
            kp_3hr: None,
            dtc: None,
        });
    }

    Ok(records)
}

// DTCFILE format: A4 I4 I5 24×I4  (total 109 chars)
// Fields: "DTC " YYYY DDD Dtc1..Dtc24
const DTCFILE_MIN_LEN: usize = 109;
const DTCFILE_YYYY: (usize, usize) = (4, 8);
const DTCFILE_DDD: (usize, usize) = (8, 13);
const DTCFILE_DTC_START: usize = 13;
const DTCFILE_DTC_WIDTH: usize = 4;
const DTCFILE_DTC_COUNT: usize = 24;

pub fn parse_dtcfile(input: &[u8]) -> Result<Vec<SpaceWeatherRecord>, SpaceWeatherError> {
    let text = str::from_utf8(input).map_err(|_| SpaceWeatherError::ParseError {
        row: 0,
        message: String::from("invalid utf-8"),
    })?;

    let mut records = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if skip_line(line) {
            continue;
        }
        let row = i + 1;
        let b = line.as_bytes();
        if b.len() < DTCFILE_MIN_LEN {
            return Err(SpaceWeatherError::ParseError {
                row,
                message: String::from("line too short"),
            });
        }

        let year: i32 = parse_int(sl(b, DTCFILE_YYYY.0, DTCFILE_YYYY.1), row, "year")?;
        let doy: u16 = parse_int(sl(b, DTCFILE_DDD.0, DTCFILE_DDD.1), row, "doy")?;
        let date = doy_to_date(year, doy, row)?;

        let mut sum: f64 = 0.0;
        for j in 0..DTCFILE_DTC_COUNT {
            let start = DTCFILE_DTC_START + j * DTCFILE_DTC_WIDTH;
            let end = start + DTCFILE_DTC_WIDTH;
            let val: i32 = parse_int(sl(b, start, end), row, "Dtc")?;
            sum += val as f64;
        }

        records.push(SpaceWeatherRecord {
            date,
            dtc: Some(sum / DTCFILE_DTC_COUNT as f64),
            f10_7: None,
            f10_7a: None,
            ap_daily: None,
            ap_3hr: None,
            kp_3hr: None,
            s10_7: None,
            m10_7: None,
            y10_7: None,
        });
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real SOLFSMY format: I6 I4 F12.1 8×F6.1 A6 (76 chars)
    //                      YYYY DDD JulDay F10  F81c S10  S81c M10  M81c Y10  Y81c Ssrc
    const SOLFSMY_FIXTURE: &str = "\
# SOLFSMY.TXT  Solar indices for JB2008
# YYYY DDD   JulianDay  F10   F81c  S10   S81c  M10   M81c  Y10   Y81c  Ssrc
  2023 166   2460111.5 150.3 148.1 120.5 118.2 115.3 113.1 118.9 116.5  0000
  2023 167   2460112.5 152.0 149.5 121.0 118.4 116.1 113.3 119.5 116.7  0000
";

    // Real DTCFILE format: A4 I4 I5 24×I4 (109 chars)
    //                      "DTC " YYYY DDD Dtc1..Dtc24
    const DTCFILE_FIXTURE: &str = "\
DTC 2023  166  31  50  50  50  44  44  44  50  50  50  44  44  44  24  24  24  24  24  24  31  31  31  31  31
DTC 2023  167  24  38  38  38  31  31  31  38  38  38  31  31  31  31  31  31  38  38  38  17  17  17  17  17
";

    // --- SOLFSMY tests ---

    #[test]
    fn solfsmy_valid_parse() {
        let records = parse_solfsmy(SOLFSMY_FIXTURE.as_bytes()).unwrap();
        assert_eq!(records.len(), 2);

        // DOY 166 = June 15
        assert_eq!(records[0].date, Date { year: 2023, month: 6, day: 15 });
        assert_eq!(records[0].f10_7, Some(150.3));
        assert_eq!(records[0].f10_7a, Some(148.1));
        assert_eq!(records[0].s10_7, Some(120.5));
        assert_eq!(records[0].m10_7, Some(115.3));
        assert_eq!(records[0].y10_7, Some(118.9));

        // DOY 167 = June 16
        assert_eq!(records[1].date, Date { year: 2023, month: 6, day: 16 });
        assert_eq!(records[1].f10_7, Some(152.0));
    }

    #[test]
    fn solfsmy_comment_skipping() {
        let input = "# comment line\n# another comment\n";
        let records = parse_solfsmy(input.as_bytes()).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn solfsmy_truncated_line() {
        let input = "  2023 166   2460111.5 150.3 148.1 120.5\n";
        let result = parse_solfsmy(input.as_bytes());
        match result {
            Err(SpaceWeatherError::ParseError { message, .. }) => {
                assert_eq!(message, "line too short");
            }
            other => panic!("expected ParseError, got {:?}", other),
        }
    }

    #[test]
    fn solfsmy_invalid_numeric() {
        let input = "  2023 166   2460111.5 XXXXX 148.1 120.5 118.2 115.3 113.1 118.9 116.5  0000\n";
        let result = parse_solfsmy(input.as_bytes());
        assert!(matches!(result, Err(SpaceWeatherError::ParseError { .. })));
    }

    #[test]
    fn solfsmy_leap_year_doy() {
        // 2024 is leap year, DOY 60 = Feb 29
        let input = "  2024  60   2460370.5 150.3 148.1 120.5 118.2 115.3 113.1 118.9 116.5  0000\n";
        let records = parse_solfsmy(input.as_bytes()).unwrap();
        assert_eq!(records[0].date, Date { year: 2024, month: 2, day: 29 });
    }

    // --- DTCFILE tests ---

    #[test]
    fn dtcfile_valid_parse() {
        let records = parse_dtcfile(DTCFILE_FIXTURE.as_bytes()).unwrap();
        assert_eq!(records.len(), 2);

        // DOY 166 = June 15
        assert_eq!(records[0].date, Date { year: 2023, month: 6, day: 15 });
        // Sum: 31+50+50+50+44+44+44+50+50+50+44+44+44+24+24+24+24+24+24+31+31+31+31+31 = 894
        let expected_mean = 894.0 / 24.0;
        assert!((records[0].dtc.unwrap() - expected_mean).abs() < 1e-10);

        // DOY 167 = June 16
        assert_eq!(records[1].date, Date { year: 2023, month: 6, day: 16 });
    }

    #[test]
    fn dtcfile_daily_mean_correctness() {
        // All 24 values = 10 → mean = 10.0
        let input = "DTC 2023  166  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10  10\n";
        let records = parse_dtcfile(input.as_bytes()).unwrap();
        assert_eq!(records[0].dtc, Some(10.0));
    }

    #[test]
    fn dtcfile_comment_skipping() {
        let input = "# comment\n\n# another\n";
        let records = parse_dtcfile(input.as_bytes()).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn dtcfile_truncated_line() {
        let input = "DTC 2023 166  10  12  11\n";
        let result = parse_dtcfile(input.as_bytes());
        match result {
            Err(SpaceWeatherError::ParseError { message, .. }) => {
                assert_eq!(message, "line too short");
            }
            other => panic!("expected ParseError, got {:?}", other),
        }
    }

    #[test]
    fn dtcfile_invalid_numeric() {
        let input = "DTC 2023  166  XX  12  11  13  15  14  12  11  10   9   8  10  12  14  16  15  13  12  11  10   9   8  10  11\n";
        let result = parse_dtcfile(input.as_bytes());
        assert!(matches!(result, Err(SpaceWeatherError::ParseError { .. })));
    }
}
