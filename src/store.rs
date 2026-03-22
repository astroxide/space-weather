use alloc::vec::Vec;

use crate::{Date, SpaceWeatherIndex, SpaceWeatherRecord};

pub struct SpaceWeatherStore {
    records: Vec<SpaceWeatherRecord>,
}

impl SpaceWeatherStore {
    pub fn new(mut records: Vec<SpaceWeatherRecord>) -> Self {
        records.sort_by_key(|r| r.date);
        let records = dedup_merge(records);
        Self { records }
    }

    pub fn merge(&mut self, other: SpaceWeatherStore) {
        let mut combined = Vec::with_capacity(self.records.len() + other.records.len());
        combined.append(&mut self.records);
        combined.extend(other.records);
        combined.sort_by_key(|r| r.date);
        self.records = dedup_merge(combined);
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn first_date(&self) -> Option<Date> {
        self.records.first().map(|r| r.date)
    }

    pub fn last_date(&self) -> Option<Date> {
        self.records.last().map(|r| r.date)
    }
}

impl SpaceWeatherIndex for SpaceWeatherStore {
    fn get(&self, date: Date) -> Option<&SpaceWeatherRecord> {
        self.records
            .binary_search_by_key(&date, |r| r.date)
            .ok()
            .map(|i| &self.records[i])
    }

    fn get_range(&self, start: Date, end: Date) -> Vec<&SpaceWeatherRecord> {
        if start > end {
            return Vec::new();
        }
        let lo = self.records.partition_point(|r| r.date < start);
        let hi = self.records.partition_point(|r| r.date <= end);
        self.records[lo..hi].iter().collect()
    }
}

fn merge_record(target: &mut SpaceWeatherRecord, source: SpaceWeatherRecord) {
    macro_rules! fill {
        ($field:ident) => {
            if target.$field.is_none() {
                target.$field = source.$field;
            }
        };
    }
    fill!(f10_7);
    fill!(f10_7a);
    fill!(ap_daily);
    fill!(ap_3hr);
    fill!(kp_3hr);
    fill!(s10_7);
    fill!(m10_7);
    fill!(y10_7);
    fill!(dtc);
}

fn dedup_merge(mut records: Vec<SpaceWeatherRecord>) -> Vec<SpaceWeatherRecord> {
    if records.len() <= 1 {
        return records;
    }
    let mut write = 0;
    for read in 1..records.len() {
        if records[write].date == records[read].date {
            let source = records[read].clone();
            merge_record(&mut records[write], source);
        } else {
            write += 1;
            if write != read {
                records.swap(write, read);
            }
        }
    }
    records.truncate(write + 1);
    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn date(y: i32, m: u8, d: u8) -> Date {
        Date {
            year: y,
            month: m,
            day: d,
        }
    }

    fn empty_record(d: Date) -> SpaceWeatherRecord {
        SpaceWeatherRecord {
            date: d,
            f10_7: None,
            f10_7a: None,
            ap_daily: None,
            ap_3hr: None,
            kp_3hr: None,
            s10_7: None,
            m10_7: None,
            y10_7: None,
            dtc: None,
        }
    }

    fn record_with(
        d: Date,
        f10_7: Option<f64>,
        ap_daily: Option<f64>,
        s10_7: Option<f64>,
    ) -> SpaceWeatherRecord {
        SpaceWeatherRecord {
            f10_7,
            ap_daily,
            s10_7,
            ..empty_record(d)
        }
    }

    // 5.1 Construction tests

    #[test]
    fn new_sorts_records() {
        let store = SpaceWeatherStore::new(vec![
            empty_record(date(2024, 3, 15)),
            empty_record(date(2024, 1, 1)),
            empty_record(date(2024, 2, 10)),
        ]);
        assert_eq!(store.records[0].date, date(2024, 1, 1));
        assert_eq!(store.records[1].date, date(2024, 2, 10));
        assert_eq!(store.records[2].date, date(2024, 3, 15));
    }

    #[test]
    fn new_merges_duplicate_dates() {
        let d = date(2024, 1, 1);
        let store = SpaceWeatherStore::new(vec![
            record_with(d, Some(150.0), None, None),
            record_with(d, None, Some(10.0), None),
        ]);
        assert_eq!(store.len(), 1);
        assert_eq!(store.records[0].f10_7, Some(150.0));
        assert_eq!(store.records[0].ap_daily, Some(10.0));
    }

    #[test]
    fn new_first_some_wins_on_duplicate() {
        let d = date(2024, 1, 1);
        let store = SpaceWeatherStore::new(vec![
            record_with(d, Some(150.0), None, None),
            record_with(d, Some(999.0), None, None),
        ]);
        assert_eq!(store.records[0].f10_7, Some(150.0));
    }

    #[test]
    fn new_empty_input() {
        let store = SpaceWeatherStore::new(Vec::new());
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    // 5.2 Exact lookup tests

    #[test]
    fn get_existing_date() {
        let d = date(2024, 6, 15);
        let store = SpaceWeatherStore::new(vec![
            empty_record(date(2024, 1, 1)),
            record_with(d, Some(140.0), None, None),
            empty_record(date(2024, 12, 31)),
        ]);
        let rec = store.get(d).unwrap();
        assert_eq!(rec.f10_7, Some(140.0));
    }

    #[test]
    fn get_missing_date() {
        let store = SpaceWeatherStore::new(vec![empty_record(date(2024, 1, 1))]);
        assert!(store.get(date(2024, 1, 2)).is_none());
    }

    // 5.3 Range query tests

    #[test]
    fn get_range_returns_inclusive() {
        let store = SpaceWeatherStore::new(vec![
            empty_record(date(2024, 1, 1)),
            empty_record(date(2024, 1, 2)),
            empty_record(date(2024, 1, 3)),
            empty_record(date(2024, 1, 4)),
            empty_record(date(2024, 1, 5)),
        ]);
        let results = store.get_range(date(2024, 1, 2), date(2024, 1, 4));
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].date, date(2024, 1, 2));
        assert_eq!(results[2].date, date(2024, 1, 4));
    }

    #[test]
    fn get_range_empty_result() {
        let store = SpaceWeatherStore::new(vec![empty_record(date(2024, 1, 1))]);
        let results = store.get_range(date(2024, 6, 1), date(2024, 6, 30));
        assert!(results.is_empty());
    }

    #[test]
    fn get_range_inverted() {
        let store = SpaceWeatherStore::new(vec![empty_record(date(2024, 1, 1))]);
        let results = store.get_range(date(2024, 12, 31), date(2024, 1, 1));
        assert!(results.is_empty());
    }

    // 5.4 Merge tests

    #[test]
    fn merge_complementary_sources() {
        let d = date(2024, 1, 1);
        let mut a = SpaceWeatherStore::new(vec![record_with(d, Some(150.0), Some(10.0), None)]);
        let b = SpaceWeatherStore::new(vec![record_with(d, None, None, Some(120.0))]);
        a.merge(b);
        assert_eq!(a.len(), 1);
        let rec = a.get(d).unwrap();
        assert_eq!(rec.f10_7, Some(150.0));
        assert_eq!(rec.ap_daily, Some(10.0));
        assert_eq!(rec.s10_7, Some(120.0));
    }

    #[test]
    fn merge_overlapping_fields_self_wins() {
        let d = date(2024, 1, 1);
        let mut a = SpaceWeatherStore::new(vec![record_with(d, Some(150.0), None, None)]);
        let b = SpaceWeatherStore::new(vec![record_with(d, Some(999.0), None, None)]);
        a.merge(b);
        assert_eq!(a.get(d).unwrap().f10_7, Some(150.0));
    }

    #[test]
    fn merge_disjoint_dates() {
        let mut a = SpaceWeatherStore::new(vec![empty_record(date(2024, 1, 1))]);
        let b = SpaceWeatherStore::new(vec![empty_record(date(2024, 6, 1))]);
        a.merge(b);
        assert_eq!(a.len(), 2);
        assert_eq!(a.first_date(), Some(date(2024, 1, 1)));
        assert_eq!(a.last_date(), Some(date(2024, 6, 1)));
    }

    // 5.5 Accessor tests

    #[test]
    fn bounds_non_empty() {
        let store = SpaceWeatherStore::new(vec![
            empty_record(date(2024, 3, 1)),
            empty_record(date(2024, 1, 1)),
            empty_record(date(2024, 12, 31)),
        ]);
        assert_eq!(store.len(), 3);
        assert!(!store.is_empty());
        assert_eq!(store.first_date(), Some(date(2024, 1, 1)));
        assert_eq!(store.last_date(), Some(date(2024, 12, 31)));
    }

    #[test]
    fn bounds_empty() {
        let store = SpaceWeatherStore::new(Vec::new());
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
        assert_eq!(store.first_date(), None);
        assert_eq!(store.last_date(), None);
    }
}
