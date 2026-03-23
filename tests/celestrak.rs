use space_weather::parsers::celestrak;
use space_weather::Date;

const SW_ALL: &[u8] = include_bytes!("fixtures/SW-All.csv");
const SW_LAST5: &[u8] = include_bytes!("fixtures/SW-Last5Years.csv");

#[test]
fn parse_sw_all() {
    let records = celestrak::parse(SW_ALL).unwrap();
    assert!(
        records.len() > 25_000,
        "expected >25k rows, got {}",
        records.len()
    );

    // first record: 1957-10-01
    let first = &records[0];
    assert_eq!(
        first.date,
        Date {
            year: 1957,
            month: 10,
            day: 1
        }
    );
    assert_eq!(first.f10_7_obs, Some(269.3));
    assert_eq!(first.ap_daily, Some(21.0));
    assert!(first.ap_3hr.is_some());
    assert!(first.kp_3hr.is_some());
}

#[test]
fn parse_sw_last5years() {
    let records = celestrak::parse(SW_LAST5).unwrap();
    assert!(
        records.len() > 1_000,
        "expected >1k rows, got {}",
        records.len()
    );

    let first = &records[0];
    assert_eq!(
        first.date,
        Date {
            year: 2021,
            month: 1,
            day: 1
        }
    );
    assert_eq!(first.f10_7_obs, Some(80.4));
    assert_eq!(first.f10_7_adj, Some(77.7));
}

#[test]
fn predicted_monthly_rows_have_empty_kp_ap() {
    let records = celestrak::parse(SW_ALL).unwrap();
    // PRM rows start at 2026-06-01 — monthly predictions with no Kp/Ap
    let predicted: Vec<_> = records.iter().filter(|r| r.date.year >= 2027).collect();
    assert!(!predicted.is_empty());
    for r in &predicted {
        assert_eq!(r.kp_3hr, None);
        assert_eq!(r.ap_3hr, None);
        assert_eq!(r.ap_daily, None);
        assert!(
            r.f10_7_obs.is_some(),
            "predicted row should still have F10.7"
        );
    }
}

#[test]
fn chronological_order() {
    let records = celestrak::parse(SW_ALL).unwrap();
    for pair in records.windows(2) {
        assert!(
            pair[0].date <= pair[1].date,
            "out of order: {:?} > {:?}",
            pair[0].date,
            pair[1].date
        );
    }
}

#[test]
fn bench_parse_sw_all() {
    use std::time::Instant;
    // warm up
    let _ = celestrak::parse(SW_ALL).unwrap();

    let iterations = 20;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = celestrak::parse(SW_ALL).unwrap();
    }
    let elapsed = start.elapsed();
    let per_iter = elapsed / iterations;
    let mb = SW_ALL.len() as f64 / 1_048_576.0;

    eprintln!(
        "SW-All.csv ({:.1} MB, {} lines): {:?}/iter, {:.0} MB/s",
        mb,
        25237,
        per_iter,
        mb / per_iter.as_secs_f64()
    );
}
