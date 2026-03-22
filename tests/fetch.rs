#![cfg(feature = "fetch-blocking")]

use space_weather::fetch::blocking;
use space_weather::fetch::{CacheHeaders, FetchResult};

#[test]
fn cache_headers_default_is_empty() {
    let ch = CacheHeaders::default();
    assert!(ch.etag.is_none());
    assert!(ch.last_modified.is_none());
}

#[test]
fn fetch_result_data_variant() {
    let result = FetchResult::Data {
        bytes: vec![1, 2, 3],
        cache: CacheHeaders::default(),
    };
    assert!(matches!(result, FetchResult::Data { .. }));
}

#[test]
fn fetch_result_not_modified_variant() {
    let result = FetchResult::NotModified;
    assert!(matches!(result, FetchResult::NotModified));
}

#[test]
#[ignore]
fn blocking_fetch_celestrak_5yr() {
    let result = blocking::fetch_celestrak_5yr(None).expect("fetch failed");
    match result {
        FetchResult::Data { bytes, cache } => {
            assert!(!bytes.is_empty());
            // CelesTrak CSV starts with a header line
            let text = String::from_utf8_lossy(&bytes);
            assert!(text.contains("F10.7"));
            // Try conditional GET with returned headers
            let result2 =
                blocking::fetch_celestrak_5yr(Some(&cache)).expect("conditional fetch failed");
            // Server may or may not support conditional GET; both outcomes valid
            assert!(matches!(
                result2,
                FetchResult::Data { .. } | FetchResult::NotModified
            ));
        }
        FetchResult::NotModified => panic!("first fetch should not return NotModified"),
    }
}

#[test]
#[ignore]
fn blocking_fetch_conditional_get_roundtrip() {
    let result = blocking::fetch_solfsmy(None).expect("initial fetch failed");
    let cache = match result {
        FetchResult::Data { cache, .. } => cache,
        FetchResult::NotModified => panic!("first fetch should return data"),
    };

    // Second fetch with cache headers
    let result2 = blocking::fetch_solfsmy(Some(&cache)).expect("conditional fetch failed");
    assert!(matches!(
        result2,
        FetchResult::Data { .. } | FetchResult::NotModified
    ));
}
