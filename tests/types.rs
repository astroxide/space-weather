use space_weather::{Date, SpaceWeatherError, SpaceWeatherRecord};

#[test]
fn date_ordering() {
    let d1 = Date {
        year: 2023,
        month: 1,
        day: 1,
    };
    let d2 = Date {
        year: 2023,
        month: 6,
        day: 15,
    };
    let d3 = Date {
        year: 2024,
        month: 1,
        day: 1,
    };
    assert!(d1 < d2);
    assert!(d2 < d3);
    assert_eq!(
        d1,
        Date {
            year: 2023,
            month: 1,
            day: 1
        }
    );
}

#[test]
fn celestrak_only_record() {
    let rec = SpaceWeatherRecord {
        date: Date {
            year: 2023,
            month: 10,
            day: 15,
        },
        f10_7_obs: Some(150.0),
        f10_7_adj: Some(145.0),
        f10_7_jb: None,
        f10_7_jb_81c: None,
        ap_daily: Some(12.0),
        ap_3hr: Some([5.0, 7.0, 9.0, 12.0, 15.0, 9.0, 6.0, 4.0]),
        kp_3hr: Some([1.7, 2.0, 2.3, 3.0, 3.3, 2.3, 1.7, 1.3]),
        s10_7: None,
        m10_7: None,
        y10_7: None,
        dtc: None,
    };
    assert!(rec.validate().is_ok());
    assert!(rec.s10_7.is_none());
}

#[test]
fn set_only_record() {
    let rec = SpaceWeatherRecord {
        date: Date {
            year: 2023,
            month: 10,
            day: 15,
        },
        f10_7_obs: None,
        f10_7_adj: None,
        f10_7_jb: None,
        f10_7_jb_81c: None,
        ap_daily: None,
        ap_3hr: None,
        kp_3hr: None,
        s10_7: Some(140.0),
        m10_7: Some(138.0),
        y10_7: Some(142.0),
        dtc: Some(10.5),
    };
    assert!(rec.validate().is_ok());
    assert!(rec.ap_daily.is_none());
}

#[test]
fn merged_record() {
    let rec = SpaceWeatherRecord {
        date: Date {
            year: 2023,
            month: 10,
            day: 15,
        },
        f10_7_obs: Some(150.0),
        f10_7_adj: Some(145.0),
        f10_7_jb: None,
        f10_7_jb_81c: None,
        ap_daily: Some(12.0),
        ap_3hr: None,
        kp_3hr: None,
        s10_7: Some(140.0),
        m10_7: Some(138.0),
        y10_7: Some(142.0),
        dtc: Some(10.5),
    };
    assert!(rec.validate().is_ok());
    assert!(rec.f10_7_obs.is_some());
    assert!(rec.s10_7.is_some());
}

#[test]
fn invalid_date_rejected() {
    let d = Date {
        year: 2023,
        month: 13,
        day: 1,
    };
    assert_eq!(d.validate(), Err(SpaceWeatherError::InvalidDate));

    let d2 = Date {
        year: 2023,
        month: 0,
        day: 1,
    };
    assert_eq!(d2.validate(), Err(SpaceWeatherError::InvalidDate));
}

#[test]
fn negative_index_rejected() {
    let rec = SpaceWeatherRecord {
        date: Date {
            year: 2023,
            month: 1,
            day: 1,
        },
        f10_7_obs: Some(-50.0),
        f10_7_adj: None,
        f10_7_jb: None,
        f10_7_jb_81c: None,
        ap_daily: None,
        ap_3hr: None,
        kp_3hr: None,
        s10_7: None,
        m10_7: None,
        y10_7: None,
        dtc: None,
    };
    assert_eq!(rec.validate(), Err(SpaceWeatherError::InvalidIndex));
}
