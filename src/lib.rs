//! Space weather indices and parsers for aerospace applications.
//!
//! Provides types, parsers, and a query store for space weather data from
//! CelesTrak and SET (Space Environment Technologies). Supports `no_std`
//! (with `alloc`). Optional features enable HTTP fetching and Python bindings.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod centered_average;
#[cfg(feature = "fetch")]
pub mod fetch;
pub mod parsers;
#[cfg(feature = "python")]
pub mod python;
pub mod store;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Calendar date (year, month, day) used as the primary key for space weather records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub fn validate(&self) -> Result<(), SpaceWeatherError> {
        if self.month < 1 || self.month > 12 || self.day < 1 || self.day > 31 {
            return Err(SpaceWeatherError::InvalidDate);
        }
        Ok(())
    }
}

/// A single day's space weather indices from one or more data sources.
///
/// Fields are `Option` because not every source provides every index.
/// Use [`store::SpaceWeatherStore::merge`] to combine records from different sources.
#[derive(Clone, Debug)]
pub struct SpaceWeatherRecord {
    pub date: Date,
    pub f10_7_obs: Option<f64>,
    pub f10_7_adj: Option<f64>,
    pub f10_7_jb: Option<f64>,
    pub f10_7_jb_81c: Option<f64>,
    pub ap_daily: Option<f64>,
    pub ap_3hr: Option<[f64; 8]>,
    pub kp_3hr: Option<[f64; 8]>,
    pub s10_7: Option<f64>,
    pub m10_7: Option<f64>,
    pub y10_7: Option<f64>,
    pub dtc: Option<f64>,
}

impl SpaceWeatherRecord {
    pub fn validate(&self) -> Result<(), SpaceWeatherError> {
        self.date.validate()?;
        for v in [
            self.f10_7_obs,
            self.f10_7_adj,
            self.f10_7_jb,
            self.f10_7_jb_81c,
            self.ap_daily,
            self.s10_7,
            self.m10_7,
            self.y10_7,
            self.dtc,
        ]
        .into_iter()
        .flatten()
        {
            if v < 0.0 {
                return Err(SpaceWeatherError::InvalidIndex);
            }
        }
        Ok(())
    }
}

/// Trait for querying space weather data by date.
pub trait SpaceWeatherIndex {
    /// Returns the record for an exact date, or `None` if not present.
    fn get(&self, date: Date) -> Option<&SpaceWeatherRecord>;

    /// Returns all records in the inclusive date range `[start, end]`.
    fn get_range(&self, start: Date, end: Date) -> Vec<&SpaceWeatherRecord>;
}

/// Errors returned by parsing, validation, and store operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpaceWeatherError {
    InvalidDate,
    InvalidIndex,
    InvalidHeader,
    InvalidWindow,
    ParseError { row: usize, message: String },
}

impl fmt::Display for SpaceWeatherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDate => write!(f, "invalid date"),
            Self::InvalidIndex => write!(f, "invalid index value"),
            Self::InvalidHeader => write!(f, "invalid or missing CSV header"),
            Self::InvalidWindow => write!(f, "window must be a positive odd number"),
            Self::ParseError { row, message } => {
                write!(f, "parse error at row {}: {}", row, message)
            }
        }
    }
}
