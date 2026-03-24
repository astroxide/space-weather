use space_weather::fetch::blocking::{fetch_celestrak_5yr, fetch_solfsmy};
use space_weather::fetch::FetchResult;
use space_weather::parsers::{celestrak, set};
use space_weather::store::SpaceWeatherStore;
use space_weather::{Date, SpaceWeatherIndex};

fn main() {
    println!("Fetching CelesTrak SW-Last5Years...");
    let celestrak_data = match fetch_celestrak_5yr(None).expect("fetch failed") {
        FetchResult::Data { bytes, .. } => bytes,
        FetchResult::NotModified => unreachable!("no cache headers sent"),
    };
    let celestrak_records = celestrak::parse(&celestrak_data).expect("parse failed");
    let mut store = SpaceWeatherStore::new(celestrak_records);
    println!("  {} CelesTrak records", store.len());

    println!("Fetching SET SOLFSMY...");
    let solfsmy_data = match fetch_solfsmy(None).expect("fetch failed") {
        FetchResult::Data { bytes, .. } => bytes,
        FetchResult::NotModified => unreachable!("no cache headers sent"),
    };
    let solfsmy_records = set::parse_solfsmy(&solfsmy_data).expect("parse failed");
    let solfsmy_store = SpaceWeatherStore::new(solfsmy_records);
    println!("  {} SET records", solfsmy_store.len());

    store.merge(solfsmy_store);
    println!(
        "Merged store: {} records ({:?} to {:?})",
        store.len(),
        store.first_date(),
        store.last_date()
    );

    let date = Date {
        year: 2023,
        month: 6,
        day: 15,
    };
    match store.get(date) {
        Some(rec) => {
            println!(
                "\nDate:      {}-{:02}-{:02}",
                date.year, date.month, date.day
            );
            println!("F10.7 obs: {:?}", rec.f10_7_obs);
            println!("Ap daily:  {:?}", rec.ap_daily);
            println!("S10.7:     {:?}", rec.s10_7);
            println!("M10.7:     {:?}", rec.m10_7);
        }
        None => println!(
            "No record for {}-{:02}-{:02}",
            date.year, date.month, date.day
        ),
    }
}
