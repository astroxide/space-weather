use space_weather::centered_average::{centered_mean, compute_for_records, DEFAULT_WINDOW};
use space_weather::parsers::celestrak;
use space_weather::{Date, SpaceWeatherError, SpaceWeatherRecord};

#[test]
fn default_window_is_81() {
    assert_eq!(DEFAULT_WINDOW, 81);
}

#[test]
fn empty_input() {
    let result = centered_mean(&[], 3).unwrap();
    assert!(result.is_empty());
}

#[test]
fn window_of_1() {
    let vals = vec![Some(5.0), None, Some(3.0)];
    let result = centered_mean(&vals, 1).unwrap();
    assert_eq!(result, vals);
}

#[test]
fn window_of_5() {
    let vals: Vec<Option<f64>> = (1..=5).map(|v| Some(v as f64)).collect();
    let result = centered_mean(&vals, 5).unwrap();
    assert_eq!(result[0], None);
    assert_eq!(result[1], None);
    assert_eq!(result[2], Some(3.0));
    assert_eq!(result[3], None);
    assert_eq!(result[4], None);
}

#[test]
fn basic_averaging() {
    let vals: Vec<Option<f64>> = (0..7).map(|v| Some(v as f64)).collect();
    let result = centered_mean(&vals, 3).unwrap();
    assert_eq!(result[0], None);
    assert_eq!(result[1], Some(1.0));
    assert_eq!(result[2], Some(2.0));
    assert_eq!(result[3], Some(3.0));
    assert_eq!(result[4], Some(4.0));
    assert_eq!(result[5], Some(5.0));
    assert_eq!(result[6], None);
}

#[test]
fn even_window_rejected() {
    assert_eq!(
        centered_mean(&[Some(1.0)], 80),
        Err(SpaceWeatherError::InvalidWindow)
    );
}

#[test]
fn zero_window_rejected() {
    assert_eq!(
        centered_mean(&[Some(1.0)], 0),
        Err(SpaceWeatherError::InvalidWindow)
    );
}

#[test]
fn none_propagation() {
    let vals = vec![Some(1.0), None, Some(3.0), Some(4.0), Some(5.0)];
    let result = centered_mean(&vals, 3).unwrap();
    assert_eq!(result[0], None); // incomplete window
    assert_eq!(result[1], None); // window contains None at index 0? No — window is [0,1,2], index 1 is None
    assert_eq!(result[2], None); // window [1,2,3] contains None at index 1
    assert_eq!(result[3], Some(4.0)); // window [2,3,4] all Some
    assert_eq!(result[4], None); // incomplete window
}

fn make_record(date_day: u8, f10_7: Option<f64>, f10_7a: Option<f64>) -> SpaceWeatherRecord {
    SpaceWeatherRecord {
        date: Date {
            year: 2023,
            month: 1,
            day: date_day,
        },
        f10_7,
        f10_7a,
        ap_daily: None,
        ap_3hr: None,
        kp_3hr: None,
        s10_7: None,
        m10_7: None,
        y10_7: None,
        dtc: None,
    }
}

#[test]
fn compute_for_records_populates_f10_7a() {
    let mut records: Vec<SpaceWeatherRecord> = (1..=5)
        .map(|d| make_record(d, Some(d as f64 * 10.0), None))
        .collect();

    compute_for_records(&mut records, 3, |r| r.f10_7, |r, v| r.f10_7a = v).unwrap();

    assert_eq!(records[0].f10_7a, None);
    assert_eq!(records[1].f10_7a, Some(20.0));
    assert_eq!(records[2].f10_7a, Some(30.0));
    assert_eq!(records[3].f10_7a, Some(40.0));
    assert_eq!(records[4].f10_7a, None);
}

#[test]
fn compute_for_records_overwrites_existing() {
    let mut records: Vec<SpaceWeatherRecord> = (1..=5)
        .map(|d| make_record(d, Some(d as f64), Some(999.0)))
        .collect();

    compute_for_records(&mut records, 3, |r| r.f10_7, |r, v| r.f10_7a = v).unwrap();

    // Edges overwritten to None, middle overwritten to computed value
    assert_eq!(records[0].f10_7a, None);
    assert_eq!(records[2].f10_7a, Some(3.0));
    assert_eq!(records[4].f10_7a, None);
}

#[test]
fn validate_against_celestrak_precomputed() {
    let csv = include_bytes!("fixtures/SW-Last5Years.csv");

    // Parse records to get daily f10_7 values
    let mut records = celestrak::parse(csv).unwrap();

    // Extract pre-computed F10.7_OBS_CENTER81 directly from CSV
    let text = std::str::from_utf8(csv).unwrap();
    let mut lines = text.lines();
    let header = lines.next().unwrap();
    let center81_col = header
        .split(',')
        .position(|c| c.trim() == "F10.7_OBS_CENTER81")
        .expect("F10.7_OBS_CENTER81 column not found");

    let precomputed: Vec<Option<f64>> = lines
        .map(|line| {
            let field = line.split(',').nth(center81_col).unwrap_or("").trim();
            if field.is_empty() || field == "99999" || field == "999.9" || field == "9999.9" {
                None
            } else {
                field.parse().ok()
            }
        })
        .collect();

    // Compute centered averages from daily f10_7
    compute_for_records(
        &mut records,
        DEFAULT_WINDOW,
        |r| r.f10_7,
        |r, v| r.f10_7a = v,
    )
    .unwrap();

    // Compare only on observed (non-predicted) data. CelesTrak's CENTER81
    // in the predicted region uses forecast values we don't have, so we
    // restrict to dates before 2026 where observed F10.7 is final.
    let cutoff = Date {
        year: 2026,
        month: 1,
        day: 1,
    };
    let mut compared = 0;
    let mut max_diff: f64 = 0.0;
    for (i, rec) in records.iter().enumerate() {
        if rec.date >= cutoff {
            break;
        }
        if let (Some(computed), Some(expected)) = (rec.f10_7a, precomputed[i]) {
            let diff = (computed - expected).abs();
            if diff > max_diff {
                max_diff = diff;
            }
            compared += 1;
            assert!(
                diff < 0.15,
                "day {}: computed={}, expected={}, diff={}",
                i,
                computed,
                expected,
                diff
            );
        }
    }
    assert!(
        compared > 100,
        "compared {} records, max_diff={:.4}",
        compared,
        max_diff
    );
}
