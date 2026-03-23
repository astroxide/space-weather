use std::collections::HashMap;

use space_weather::parsers::{celestrak, set};

const SW_LAST5: &[u8] = include_bytes!("fixtures/SW-Last5Years.csv");
const SOLFSMY: &[u8] = include_bytes!("fixtures/SOLFSMY.TXT");

#[test]
fn f10_7_divergence_across_sources() {
    let celestrak_records = celestrak::parse(SW_LAST5).unwrap();
    let set_records = set::parse_solfsmy(SOLFSMY).unwrap();

    let set_by_date: HashMap<_, _> = set_records
        .iter()
        .filter_map(|r| r.f10_7_jb.map(|v| (r.date, v)))
        .collect();

    let mut compared = 0;
    let mut max_diff: f64 = 0.0;

    for rec in &celestrak_records {
        let obs = match rec.f10_7_obs {
            Some(v) => v,
            None => continue,
        };
        let jb = match set_by_date.get(&rec.date) {
            Some(&v) => v,
            None => continue,
        };

        let diff = (obs - jb).abs();
        if diff > max_diff {
            max_diff = diff;
        }
        if diff > 0.1 {
            eprintln!(
                "f10_7 divergence {:?}: obs={}, jb={}, diff={:.1}",
                rec.date, obs, jb, diff
            );
        }
        compared += 1;
    }

    assert!(
        compared >= 100,
        "only {} overlapping dates found (need >=100)",
        compared,
    );
    eprintln!(
        "cross-validated {} dates, max_diff={:.1}",
        compared, max_diff
    );
}
