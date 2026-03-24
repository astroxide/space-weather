use space_weather::parsers::celestrak;
use space_weather::store::SpaceWeatherStore;
use space_weather::{Date, SpaceWeatherIndex};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/fixtures/SW-Last5Years.csv".into());

    let bytes = std::fs::read(&path).expect("failed to read CSV file");
    let records = celestrak::parse(&bytes).expect("failed to parse CSV");
    let store = SpaceWeatherStore::new(records);

    println!(
        "Loaded {} records ({:?} to {:?})",
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
            println!("Date:      {}-{:02}-{:02}", date.year, date.month, date.day);
            println!("F10.7 obs: {:?}", rec.f10_7_obs);
            println!("F10.7 adj: {:?}", rec.f10_7_adj);
            println!("Ap daily:  {:?}", rec.ap_daily);
        }
        None => println!(
            "No record for {}-{:02}-{:02}",
            date.year, date.month, date.day
        ),
    }
}
