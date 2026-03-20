use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::str;

use crate::{Date, SpaceWeatherError, SpaceWeatherRecord};

const SENTINELS: &[&str] = &["99999", "999.9", "9999.9"];

fn is_missing(field: &str) -> bool {
    let s = field.trim();
    s.is_empty() || SENTINELS.contains(&s)
}

fn parse_f64_opt(field: &str) -> Option<f64> {
    if is_missing(field) {
        None
    } else {
        field.trim().parse().ok()
    }
}

fn parse_date(field: &str) -> Result<Date, SpaceWeatherError> {
    let s = field.trim();
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err(SpaceWeatherError::InvalidDate);
    }
    let year: i32 = parts[0]
        .parse()
        .map_err(|_| SpaceWeatherError::InvalidDate)?;
    let month: u8 = parts[1]
        .parse()
        .map_err(|_| SpaceWeatherError::InvalidDate)?;
    let day: u8 = parts[2]
        .parse()
        .map_err(|_| SpaceWeatherError::InvalidDate)?;
    let date = Date { year, month, day };
    date.validate()?;
    Ok(date)
}

fn build_column_map(header_line: &str) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for (i, col) in header_line.split(',').enumerate() {
        map.insert(String::from(col.trim()), i);
    }
    map
}

fn col_index(cols: &BTreeMap<String, usize>, name: &str) -> Option<usize> {
    cols.get(name).copied()
}

fn require_col(cols: &BTreeMap<String, usize>, name: &str) -> Result<usize, SpaceWeatherError> {
    col_index(cols, name).ok_or(SpaceWeatherError::InvalidHeader)
}

fn parse_array_8(fields: &[&str], indices: &[usize; 8]) -> Option<[f64; 8]> {
    let mut arr = [0.0; 8];
    for (i, &idx) in indices.iter().enumerate() {
        let field = fields.get(idx).copied().unwrap_or("");
        match parse_f64_opt(field) {
            Some(v) => arr[i] = v,
            None => return None,
        }
    }
    Some(arr)
}

fn resolve_col(cols: &BTreeMap<String, usize>, names: &[&str]) -> Result<usize, SpaceWeatherError> {
    for name in names {
        if let Some(&idx) = cols.get(*name) {
            return Ok(idx);
        }
    }
    Err(SpaceWeatherError::InvalidHeader)
}

fn resolve_array_cols(
    cols: &BTreeMap<String, usize>,
    prefix: &str,
    count: usize,
) -> Result<[usize; 8], SpaceWeatherError> {
    let mut indices = [0usize; 8];
    for i in 0..count {
        let name = alloc::format!("{}{}", prefix, i + 1);
        indices[i] = require_col(cols, &name)?;
    }
    Ok(indices)
}

struct ColumnLayout {
    date: usize,
    f10_7: usize,
    f10_7a: usize,
    ap_daily: usize,
    ap_3hr: [usize; 8],
    kp_3hr: [usize; 8],
}

fn resolve_columns(cols: &BTreeMap<String, usize>) -> Result<ColumnLayout, SpaceWeatherError> {
    Ok(ColumnLayout {
        date: require_col(cols, "DATE")?,
        f10_7: resolve_col(cols, &["F10.7_OBS", "F10.7"])?,
        f10_7a: resolve_col(cols, &["F10.7_ADJ", "F10.7A"])?,
        ap_daily: resolve_col(cols, &["AP_AVG", "AP"])?,
        ap_3hr: resolve_array_cols(cols, "AP", 8)?,
        kp_3hr: resolve_array_cols(cols, "KP", 8)?,
    })
}

fn parse_row(
    fields: &[&str],
    layout: &ColumnLayout,
    row_num: usize,
) -> Result<SpaceWeatherRecord, SpaceWeatherError> {
    let date_field = fields.get(layout.date).copied().unwrap_or("");
    let date = parse_date(date_field).map_err(|_| SpaceWeatherError::ParseError {
        row: row_num,
        message: String::from("invalid date"),
    })?;

    Ok(SpaceWeatherRecord {
        date,
        f10_7: parse_f64_opt(fields.get(layout.f10_7).copied().unwrap_or("")),
        f10_7a: parse_f64_opt(fields.get(layout.f10_7a).copied().unwrap_or("")),
        ap_daily: parse_f64_opt(fields.get(layout.ap_daily).copied().unwrap_or("")),
        ap_3hr: parse_array_8(fields, &layout.ap_3hr),
        kp_3hr: parse_array_8(fields, &layout.kp_3hr),
        s10_7: None,
        m10_7: None,
        y10_7: None,
        dtc: None,
    })
}

pub fn parse(input: &[u8]) -> Result<Vec<SpaceWeatherRecord>, SpaceWeatherError> {
    let text = str::from_utf8(input).map_err(|_| SpaceWeatherError::InvalidHeader)?;
    let mut lines = text.lines();

    let header = lines.next().ok_or(SpaceWeatherError::InvalidHeader)?;
    let cols = build_column_map(header);
    let layout = resolve_columns(&cols)?;

    let mut records = Vec::new();
    for (i, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        records.push(parse_row(&fields, &layout, i + 2)?);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "DATE,BSRN,ND,KP1,KP2,KP3,KP4,KP5,KP6,KP7,KP8,KP_SUM,AP1,AP2,AP3,AP4,AP5,AP6,AP7,AP8,AP_AVG,CP,C9,ISN,F10.7_OBS,F10.7_ADJ,F10.7_DATA_TYPE,F10.7_OBS_CENTER81,F10.7_OBS_LAST81,F10.7_ADJ_CENTER81,F10.7_ADJ_LAST81";

    fn sample_row(date: &str, f107: &str, f107a: &str, ap_daily: &str) -> String {
        alloc::format!(
            "{date},2514,25,2.0,3.3,1.7,2.3,4.0,5.0,2.7,1.0,22.0,7,12,5,9,27,39,12,4,{ap_daily},0.8,3,135,{f107},{f107a},0,150.2,148.9,147.1,146.0"
        )
    }

    fn make_csv(rows: &[&str]) -> String {
        let mut csv = String::from(HEADER);
        for row in rows {
            csv.push('\n');
            csv.push_str(row);
        }
        csv
    }

    #[test]
    fn parse_valid_multi_row() {
        let r1 = sample_row("2023-06-15", "150.3", "148.1", "14");
        let r2 = sample_row("2023-06-16", "152.0", "149.5", "12");
        let csv = make_csv(&[&r1, &r2]);
        let records = parse(csv.as_bytes()).unwrap();
        assert_eq!(records.len(), 2);

        assert_eq!(
            records[0].date,
            Date {
                year: 2023,
                month: 6,
                day: 15
            }
        );
        assert_eq!(records[0].f10_7, Some(150.3));
        assert_eq!(records[0].f10_7a, Some(148.1));
        assert_eq!(records[0].ap_daily, Some(14.0));
        assert_eq!(
            records[0].ap_3hr,
            Some([7.0, 12.0, 5.0, 9.0, 27.0, 39.0, 12.0, 4.0])
        );
        assert_eq!(
            records[0].kp_3hr,
            Some([2.0, 3.3, 1.7, 2.3, 4.0, 5.0, 2.7, 1.0])
        );

        assert_eq!(
            records[1].date,
            Date {
                year: 2023,
                month: 6,
                day: 16
            }
        );
    }

    #[test]
    fn parse_missing_sentinel_99999() {
        let row = sample_row("2023-06-15", "99999", "148.1", "14");
        let csv = make_csv(&[&row]);
        let records = parse(csv.as_bytes()).unwrap();
        assert_eq!(records[0].f10_7, None);
        assert_eq!(records[0].f10_7a, Some(148.1));
    }

    #[test]
    fn parse_missing_empty_field() {
        let row = sample_row("2023-06-15", "", "148.1", "14");
        let csv = make_csv(&[&row]);
        let records = parse(csv.as_bytes()).unwrap();
        assert_eq!(records[0].f10_7, None);
    }

    #[test]
    fn parse_missing_999_9() {
        let row = sample_row("2023-06-15", "999.9", "148.1", "14");
        let csv = make_csv(&[&row]);
        let records = parse(csv.as_bytes()).unwrap();
        assert_eq!(records[0].f10_7, None);
    }

    #[test]
    fn parse_invalid_header() {
        let csv = b"NOT,A,VALID,HEADER\n2023-06-15,1,2,3";
        let result = parse(csv);
        assert!(matches!(result, Err(SpaceWeatherError::InvalidHeader)));
    }

    #[test]
    fn parse_empty_file_header_only() {
        let csv = HEADER.as_bytes();
        let records = parse(csv).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn parse_malformed_date_returns_error_with_row() {
        let csv = make_csv(&["BAD-DATE,2514,25,2.0,3.3,1.7,2.3,4.0,5.0,2.7,1.0,22.0,7,12,5,9,27,39,12,4,14,0.8,3,135,150.3,148.1,0,150.2,148.9,147.1,146.0"]);
        let result = parse(csv.as_bytes());
        match result {
            Err(SpaceWeatherError::ParseError { row, .. }) => assert_eq!(row, 2),
            other => panic!("expected ParseError, got {:?}", other),
        }
    }

    #[test]
    fn parse_ap_3hr_missing_one_gives_none() {
        let row = "2023-06-15,2514,25,2.0,3.3,1.7,2.3,4.0,5.0,2.7,1.0,22.0,7,12,5,9,27,39,12,,14,0.8,3,135,150.3,148.1,0,150.2,148.9,147.1,146.0";
        let csv = make_csv(&[&row]);
        let records = parse(csv.as_bytes()).unwrap();
        assert_eq!(records[0].ap_3hr, None);
    }
}
