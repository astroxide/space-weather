# space-weather

[![crates.io](https://img.shields.io/crates/v/space-weather.svg)](https://crates.io/crates/space-weather)
[![PyPI](https://img.shields.io/pypi/v/astroxide-space-weather.svg)](https://pypi.org/project/astroxide-space-weather/)
[![docs.rs](https://docs.rs/space-weather/badge.svg)](https://docs.rs/space-weather)
[![CI](https://github.com/astroxide/space-weather/actions/workflows/ci.yml/badge.svg)](https://github.com/astroxide/space-weather/actions/workflows/ci.yml)

Space weather indices and parsers for aerospace applications. `no_std` compatible (with `alloc`).

Parses space weather data from [CelesTrak](https://celestrak.org) and
[SET (Space Environment Technologies)](https://sol.spacenvironment.net),
stores it in an efficient in-memory index, and supports date-based queries.

## Quick Start (Rust)

```rust
use space_weather::parsers::celestrak;
use space_weather::store::SpaceWeatherStore;
use space_weather::{Date, SpaceWeatherIndex};

// Parse a CelesTrak CSV file
let bytes = std::fs::read("SW-Last5Years.csv").unwrap();
let records = celestrak::parse(&bytes).unwrap();
let store = SpaceWeatherStore::new(records);

// Query by date
let date = Date { year: 2023, month: 6, day: 15 };
if let Some(record) = store.get(date) {
    println!("F10.7 observed: {:?}", record.f10_7_obs);
    println!("Ap daily:       {:?}", record.ap_daily);
}
```

## Quick Start (Python)

Requires the `python` feature (built via [maturin](https://www.maturin.rs/)).

```python
from space_weather import SpaceWeather

sw = SpaceWeather.from_celestrak_csv("SW-Last5Years.csv")

from datetime import date
record = sw.get(date(2023, 6, 15))
print(record["f10_7_obs"], record["ap_daily"])

# NumPy export for bulk analysis
arrays = sw.to_numpy()
print(arrays["f10_7_obs"].mean())
```

## Feature Flags

| Flag             | Description                                                 | Dependencies       |
| ---------------- | ----------------------------------------------------------- | ------------------ |
| `std`            | Enables `std` library support (off by default for `no_std`) | —                  |
| `fetch`          | Async HTTP fetching for CelesTrak and SET files             | `reqwest` (rustls) |
| `fetch-blocking` | Blocking HTTP fetching                                      | `reqwest/blocking` |
| `python`         | Python bindings via PyO3                                    | `pyo3`, `numpy`    |

## Data Sources

| Source    | File                              | Indices                                                 |
| --------- | --------------------------------- | ------------------------------------------------------- |
| CelesTrak | `SW-Last5Years.csv`, `SW-All.csv` | F10.7 (observed/adjusted), Ap (daily + 3-hr), Kp (3-hr) |
| SET       | `SOLFSMY.TXT`                     | F10.7 (JB2008), S10.7, M10.7, Y10.7 + 81-day averages   |
| SET       | `DTCFILE.TXT`                     | Dtc temperature correction coefficients                 |

Use [`SpaceWeatherStore::merge`](https://docs.rs/space-weather) to combine
records from multiple sources into a single queryable store.

## License

MIT OR Apache-2.0
