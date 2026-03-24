//! Async fetch implementations.

use super::{
    extract_cache_headers, CacheHeaders, FetchError, FetchResult, CELESTRAK_SW_ALL_URL,
    CELESTRAK_SW_LAST5YEARS_URL, SET_DTCFILE_URL, SET_SOLFSMY_URL,
};
use reqwest::{header, Client, StatusCode};

/// Fetches raw bytes from `url`, optionally using cached ETag/Last-Modified headers.
pub async fn fetch_url(url: &str, cache: Option<&CacheHeaders>) -> Result<FetchResult, FetchError> {
    let client = Client::new();
    let mut req = client.get(url);

    if let Some(ch) = cache {
        if let Some(etag) = &ch.etag {
            req = req.header(header::IF_NONE_MATCH, etag);
        }
        if let Some(lm) = &ch.last_modified {
            req = req.header(header::IF_MODIFIED_SINCE, lm);
        }
    }

    let resp = req.send().await?;
    let status = resp.status();

    if status == StatusCode::NOT_MODIFIED {
        return Ok(FetchResult::NotModified);
    }

    if !status.is_success() {
        return Err(FetchError::Status(status));
    }

    let cache_out = extract_cache_headers(resp.headers());
    let bytes = resp.bytes().await?.to_vec();

    Ok(FetchResult::Data {
        bytes,
        cache: cache_out,
    })
}

/// Fetches the CelesTrak last-5-years space weather CSV.
pub async fn fetch_celestrak_5yr(cache: Option<&CacheHeaders>) -> Result<FetchResult, FetchError> {
    fetch_url(CELESTRAK_SW_LAST5YEARS_URL, cache).await
}

/// Fetches the CelesTrak full-history space weather CSV.
pub async fn fetch_celestrak_all(cache: Option<&CacheHeaders>) -> Result<FetchResult, FetchError> {
    fetch_url(CELESTRAK_SW_ALL_URL, cache).await
}

/// Fetches the SET SOLFSMY.TXT file.
pub async fn fetch_solfsmy(cache: Option<&CacheHeaders>) -> Result<FetchResult, FetchError> {
    fetch_url(SET_SOLFSMY_URL, cache).await
}

/// Fetches the SET DTCFILE.TXT file.
pub async fn fetch_dtcfile(cache: Option<&CacheHeaders>) -> Result<FetchResult, FetchError> {
    fetch_url(SET_DTCFILE_URL, cache).await
}
