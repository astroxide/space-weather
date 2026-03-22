use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::parsers::{celestrak, set};
use crate::store::SpaceWeatherStore;
use crate::{Date, SpaceWeatherIndex, SpaceWeatherRecord};

fn date_to_py(py: Python<'_>, d: Date) -> PyResult<PyObject> {
    let datetime = py.import("datetime")?;
    let date_cls = datetime.getattr("date")?;
    Ok(date_cls.call1((d.year, d.month, d.day))?.into())
}

fn py_to_date(obj: &Bound<'_, PyAny>) -> PyResult<Date> {
    let year: i32 = obj.getattr("year")?.extract()?;
    let month: u8 = obj.getattr("month")?.extract()?;
    let day: u8 = obj.getattr("day")?.extract()?;
    Ok(Date { year, month, day })
}

fn record_to_dict<'py>(py: Python<'py>, rec: &SpaceWeatherRecord) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("date", date_to_py(py, rec.date)?)?;
    d.set_item("f10_7", rec.f10_7)?;
    d.set_item("f10_7a", rec.f10_7a)?;
    d.set_item("ap_daily", rec.ap_daily)?;
    d.set_item("ap_3hr", rec.ap_3hr.map(|a| a.to_vec()))?;
    d.set_item("kp_3hr", rec.kp_3hr.map(|a| a.to_vec()))?;
    d.set_item("s10_7", rec.s10_7)?;
    d.set_item("m10_7", rec.m10_7)?;
    d.set_item("y10_7", rec.y10_7)?;
    d.set_item("dtc", rec.dtc)?;
    Ok(d)
}

fn load_file(path: &str) -> PyResult<Vec<u8>> {
    std::fs::read(path).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

fn parse_err(e: crate::SpaceWeatherError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(e.to_string())
}

#[pyclass]
pub struct SpaceWeather {
    store: SpaceWeatherStore,
}

#[pymethods]
impl SpaceWeather {
    #[staticmethod]
    fn from_celestrak_csv(path: &str) -> PyResult<Self> {
        let bytes = load_file(path)?;
        let records = celestrak::parse(&bytes).map_err(parse_err)?;
        Ok(Self {
            store: SpaceWeatherStore::new(records),
        })
    }

    #[staticmethod]
    fn from_set_solfsmy(path: &str) -> PyResult<Self> {
        let bytes = load_file(path)?;
        let records = set::parse_solfsmy(&bytes).map_err(parse_err)?;
        Ok(Self {
            store: SpaceWeatherStore::new(records),
        })
    }

    #[staticmethod]
    fn from_set_dtcfile(path: &str) -> PyResult<Self> {
        let bytes = load_file(path)?;
        let records = set::parse_dtcfile(&bytes).map_err(parse_err)?;
        Ok(Self {
            store: SpaceWeatherStore::new(records),
        })
    }

    #[cfg(feature = "fetch-blocking")]
    #[staticmethod]
    fn from_url(url: &str) -> PyResult<Self> {
        use crate::fetch::blocking::fetch_url;
        use crate::fetch::FetchResult;

        let result = fetch_url(url, None)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let bytes = match result {
            FetchResult::Data { bytes, .. } => bytes,
            FetchResult::NotModified => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "server returned 304 Not Modified",
                ));
            }
        };

        let url_lower = url.to_lowercase();
        let records = if url_lower.contains("solfsmy") {
            set::parse_solfsmy(&bytes).map_err(parse_err)?
        } else if url_lower.contains("dtcfile") {
            set::parse_dtcfile(&bytes).map_err(parse_err)?
        } else {
            celestrak::parse(&bytes).map_err(parse_err)?
        };

        Ok(Self {
            store: SpaceWeatherStore::new(records),
        })
    }

    fn get(&self, py: Python<'_>, date: &Bound<'_, PyAny>) -> PyResult<Option<PyObject>> {
        let d = py_to_date(date)?;
        match self.store.get(d) {
            Some(rec) => Ok(Some(record_to_dict(py, rec)?.into())),
            None => Ok(None),
        }
    }

    fn get_range<'py>(
        &self,
        py: Python<'py>,
        start: &Bound<'py, PyAny>,
        end: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let s = py_to_date(start)?;
        let e = py_to_date(end)?;
        let records = self.store.get_range(s, e);
        let dicts: Vec<PyObject> = records
            .iter()
            .map(|r| record_to_dict(py, r).map(|d| d.into()))
            .collect::<PyResult<_>>()?;
        PyList::new(py, dicts)
    }

    fn merge(&mut self, other: &mut SpaceWeather) {
        let taken = std::mem::replace(&mut other.store, SpaceWeatherStore::new(Vec::new()));
        self.store.merge(taken);
    }

    fn __len__(&self) -> usize {
        self.store.len()
    }

    fn len(&self) -> usize {
        self.store.len()
    }

    fn date_range(&self, py: Python<'_>) -> PyResult<Option<(PyObject, PyObject)>> {
        match (self.store.first_date(), self.store.last_date()) {
            (Some(first), Some(last)) => Ok(Some((date_to_py(py, first)?, date_to_py(py, last)?))),
            _ => Ok(None),
        }
    }

    fn to_numpy<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let np = py.import("numpy")?;
        let n = self.store.len();
        let range = match (self.store.first_date(), self.store.last_date()) {
            (Some(first), Some(last)) => self.store.get_range(first, last),
            _ => Vec::new(),
        };

        let mut dates: Vec<PyObject> = Vec::with_capacity(n);
        let mut f10_7 = vec![f64::NAN; n];
        let mut f10_7a = vec![f64::NAN; n];
        let mut ap_daily = vec![f64::NAN; n];
        let mut s10_7 = vec![f64::NAN; n];
        let mut m10_7 = vec![f64::NAN; n];
        let mut y10_7 = vec![f64::NAN; n];
        let mut dtc = vec![f64::NAN; n];
        let mut ap_3hr = vec![f64::NAN; n * 8];
        let mut kp_3hr = vec![f64::NAN; n * 8];

        for (i, rec) in range.iter().enumerate() {
            dates.push(date_to_py(py, rec.date)?);
            if let Some(v) = rec.f10_7 {
                f10_7[i] = v;
            }
            if let Some(v) = rec.f10_7a {
                f10_7a[i] = v;
            }
            if let Some(v) = rec.ap_daily {
                ap_daily[i] = v;
            }
            if let Some(v) = rec.s10_7 {
                s10_7[i] = v;
            }
            if let Some(v) = rec.m10_7 {
                m10_7[i] = v;
            }
            if let Some(v) = rec.y10_7 {
                y10_7[i] = v;
            }
            if let Some(v) = rec.dtc {
                dtc[i] = v;
            }
            if let Some(arr) = rec.ap_3hr {
                for j in 0..8 {
                    ap_3hr[i * 8 + j] = arr[j];
                }
            }
            if let Some(arr) = rec.kp_3hr {
                for j in 0..8 {
                    kp_3hr[i * 8 + j] = arr[j];
                }
            }
        }

        let result = PyDict::new(py);
        let date_array = np.call_method1("array", (dates,))?;
        result.set_item("date", date_array)?;

        macro_rules! set_array {
            ($name:expr, $vec:expr) => {
                let arr = numpy::PyArray1::from_vec(py, $vec);
                result.set_item($name, arr)?;
            };
        }

        set_array!("f10_7", f10_7);
        set_array!("f10_7a", f10_7a);
        set_array!("ap_daily", ap_daily);
        set_array!("s10_7", s10_7);
        set_array!("m10_7", m10_7);
        set_array!("y10_7", y10_7);
        set_array!("dtc", dtc);

        let ap_3hr_arr = numpy::PyArray2::from_vec2(
            py,
            &ap_3hr.chunks(8).map(|c| c.to_vec()).collect::<Vec<_>>(),
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        result.set_item("ap_3hr", ap_3hr_arr)?;

        let kp_3hr_arr = numpy::PyArray2::from_vec2(
            py,
            &kp_3hr.chunks(8).map(|c| c.to_vec()).collect::<Vec<_>>(),
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        result.set_item("kp_3hr", kp_3hr_arr)?;

        Ok(result)
    }
}

#[pymodule]
fn space_weather(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SpaceWeather>()?;
    Ok(())
}
