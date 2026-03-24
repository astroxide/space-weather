# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-24

### Added

- Core types: `Date`, `SpaceWeatherRecord`, `SpaceWeatherIndex` trait, `SpaceWeatherError`
- CelesTrak CSV parser (`SW-Last5Years.csv`, `SW-All.csv` formats)
- SET fixed-width parsers (`SOLFSMY.TXT`, `DTCFILE.TXT`)
- 81-day centered running mean computation
- In-memory `SpaceWeatherStore` with date lookup, range queries, and multi-source merge
- HTTP fetch with conditional GET support (`fetch` and `fetch-blocking` features)
- Python bindings via PyO3 with NumPy export (`python` feature)
- `no_std` support (with `alloc`)
