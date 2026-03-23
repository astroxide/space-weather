use space_weather::parsers::set;
use space_weather::Date;

const SOLFSMY: &[u8] = include_bytes!("fixtures/SOLFSMY.TXT");
const DTCFILE: &[u8] = include_bytes!("fixtures/DTCFILE.TXT");

#[test]
fn parse_solfsmy_all() {
    let records = set::parse_solfsmy(SOLFSMY).unwrap();
    assert!(
        records.len() > 10_000,
        "expected >10k rows, got {}",
        records.len()
    );

    // first record: 1997-01-01 (DOY 1)
    let first = &records[0];
    assert_eq!(
        first.date,
        Date {
            year: 1997,
            month: 1,
            day: 1
        }
    );
    assert_eq!(first.f10_7_jb, Some(72.4));
    assert_eq!(first.f10_7_jb_81c, Some(78.0));
    assert_eq!(first.s10_7, Some(74.0));
    assert_eq!(first.m10_7, Some(65.4));
    assert_eq!(first.y10_7, Some(61.9));
}

#[test]
fn parse_dtcfile_all() {
    let records = set::parse_dtcfile(DTCFILE).unwrap();
    assert!(
        records.len() > 10_000,
        "expected >10k rows, got {}",
        records.len()
    );

    // first record: 1997-01-01 (DOY 1)
    let first = &records[0];
    assert_eq!(
        first.date,
        Date {
            year: 1997,
            month: 1,
            day: 1
        }
    );
    assert!(first.dtc.is_some());
}

#[test]
fn solfsmy_chronological_order() {
    let records = set::parse_solfsmy(SOLFSMY).unwrap();
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
fn dtcfile_chronological_order() {
    let records = set::parse_dtcfile(DTCFILE).unwrap();
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
fn bench_parse_solfsmy() {
    use std::time::Instant;
    let _ = set::parse_solfsmy(SOLFSMY).unwrap();

    let iterations = 20;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = set::parse_solfsmy(SOLFSMY).unwrap();
    }
    let elapsed = start.elapsed();
    let per_iter = elapsed / iterations;
    let mb = SOLFSMY.len() as f64 / 1_048_576.0;

    eprintln!(
        "SOLFSMY.TXT ({:.2} MB): {:?}/iter, {:.0} MB/s",
        mb,
        per_iter,
        mb / per_iter.as_secs_f64()
    );
}

#[test]
fn bench_parse_dtcfile() {
    use std::time::Instant;
    let _ = set::parse_dtcfile(DTCFILE).unwrap();

    let iterations = 20;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = set::parse_dtcfile(DTCFILE).unwrap();
    }
    let elapsed = start.elapsed();
    let per_iter = elapsed / iterations;
    let mb = DTCFILE.len() as f64 / 1_048_576.0;

    eprintln!(
        "DTCFILE.TXT ({:.2} MB): {:?}/iter, {:.0} MB/s",
        mb,
        per_iter,
        mb / per_iter.as_secs_f64()
    );
}
