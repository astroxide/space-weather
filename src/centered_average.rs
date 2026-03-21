use alloc::vec;
use alloc::vec::Vec;

use crate::{SpaceWeatherError, SpaceWeatherRecord};

pub const DEFAULT_WINDOW: usize = 81;

pub fn centered_mean(
    values: &[Option<f64>],
    window: usize,
) -> Result<Vec<Option<f64>>, SpaceWeatherError> {
    if window == 0 || window.is_multiple_of(2) {
        return Err(SpaceWeatherError::InvalidWindow);
    }

    let n = values.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    let half = window / 2;
    let mut result = vec![None; n];

    for (i, slot) in result[half..n.saturating_sub(half)].iter_mut().enumerate() {
        let i = i + half;
        if i + half >= n {
            break;
        }
        let start = i - half;
        let end = i + half + 1;
        let mut sum = 0.0;
        let mut all_some = true;
        for v in &values[start..end] {
            match v {
                Some(x) => sum += x,
                None => {
                    all_some = false;
                    break;
                }
            }
        }
        if all_some {
            *slot = Some(sum / window as f64);
        }
    }

    Ok(result)
}

pub fn compute_for_records<G, S>(
    records: &mut [SpaceWeatherRecord],
    window: usize,
    get: G,
    set: S,
) -> Result<(), SpaceWeatherError>
where
    G: Fn(&SpaceWeatherRecord) -> Option<f64>,
    S: Fn(&mut SpaceWeatherRecord, Option<f64>),
{
    let daily: Vec<Option<f64>> = records.iter().map(&get).collect();
    let averaged = centered_mean(&daily, window)?;
    for (rec, avg) in records.iter_mut().zip(averaged) {
        set(rec, avg);
    }
    Ok(())
}
