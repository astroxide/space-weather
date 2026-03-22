import datetime
import os

import numpy as np
import pytest
import space_weather

FIXTURES = os.path.join(os.path.dirname(__file__), "..", "fixtures")


class TestCelestrakLoad:
    def test_load_and_query(self):
        sw = space_weather.SpaceWeather.from_celestrak_csv(
            os.path.join(FIXTURES, "SW-Last5Years.csv")
        )
        assert len(sw) > 0

        dr = sw.date_range()
        assert dr is not None
        assert isinstance(dr[0], datetime.date)

        rec = sw.get(dr[0])
        assert rec is not None
        assert rec["date"] == dr[0]
        assert "f10_7" in rec
        assert "ap_daily" in rec
        assert "ap_3hr" in rec
        assert "kp_3hr" in rec

    def test_missing_date_returns_none(self):
        sw = space_weather.SpaceWeather.from_celestrak_csv(
            os.path.join(FIXTURES, "SW-Last5Years.csv")
        )
        assert sw.get(datetime.date(1800, 1, 1)) is None

    def test_get_range(self):
        sw = space_weather.SpaceWeather.from_celestrak_csv(
            os.path.join(FIXTURES, "SW-Last5Years.csv")
        )
        dr = sw.date_range()
        results = sw.get_range(dr[0], dr[0])
        assert len(results) == 1
        assert results[0]["date"] == dr[0]

    def test_invalid_file_raises(self):
        with pytest.raises(ValueError):
            space_weather.SpaceWeather.from_celestrak_csv("/nonexistent/file.csv")


class TestSetLoadAndMerge:
    def test_load_solfsmy(self):
        sw = space_weather.SpaceWeather.from_set_solfsmy(
            os.path.join(FIXTURES, "SOLFSMY.TXT")
        )
        assert len(sw) > 0
        dr = sw.date_range()
        rec = sw.get(dr[0])
        assert rec is not None
        assert "s10_7" in rec

    def test_load_dtcfile(self):
        sw = space_weather.SpaceWeather.from_set_dtcfile(
            os.path.join(FIXTURES, "DTCFILE.TXT")
        )
        assert len(sw) > 0

    def test_merge(self):
        ct = space_weather.SpaceWeather.from_celestrak_csv(
            os.path.join(FIXTURES, "SW-Last5Years.csv")
        )
        sol = space_weather.SpaceWeather.from_set_solfsmy(
            os.path.join(FIXTURES, "SOLFSMY.TXT")
        )
        ct_len_before = len(ct)
        ct.merge(sol)
        assert len(ct) >= ct_len_before


class TestNumpy:
    def test_shapes_and_dtypes(self):
        sw = space_weather.SpaceWeather.from_celestrak_csv(
            os.path.join(FIXTURES, "SW-Last5Years.csv")
        )
        n = len(sw)
        arrays = sw.to_numpy()

        assert arrays["f10_7"].shape == (n,)
        assert arrays["f10_7"].dtype == np.float64
        assert arrays["ap_3hr"].shape == (n, 8)
        assert arrays["kp_3hr"].shape == (n, 8)
        assert len(arrays["date"]) == n

    def test_nan_for_missing(self):
        sw = space_weather.SpaceWeather.from_set_solfsmy(
            os.path.join(FIXTURES, "SOLFSMY.TXT")
        )
        arrays = sw.to_numpy()
        # SOLFSMY has no ap_daily data, so all should be NaN
        assert np.all(np.isnan(arrays["ap_daily"]))
