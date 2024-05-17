mod octopus;
mod powerscrape;
mod cli;
mod database;
mod data;

use data::PowerUsageCsvFormat;

use chrono::{DateTime, Utc};
use powerscrape::{start_power_scrape_thread, PowerUsage};
use std::{fs::OpenOptions, io::Write};
use std::sync::mpsc;

use database::Database;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let args = cli::get_args();    
    let (tx, rx) = mpsc::channel::<PowerUsage>();

    let database = Database::new(args.database)?;

    database.clone().ensure_schema().await?;

    let power_scrape_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./powerusage.csv")
        .unwrap();

    start_power_scrape_thread(powerscrape::Configuration {
        client_id: "uk.half-shot.autopowerbill".to_string(),
        port: args.port,
        host: args.host,
        topic: args.topic.to_string(),
        file: power_scrape_file,
    }, tx);

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./costfile.csv")
        .unwrap();
    // Fetch prices from Octopus
    let mut octopus = octopus::Octopus::new(args.product_code, args.tariff_code);
    // Pre-cache the data.
    octopus.get_price_for_period(Utc::now(), Utc::now()).await?;
    println!("Fetched prices from Octopus");

    let mut last_power_reading: f32 = 0.0;
    let mut last_date: DateTime<Utc> = DateTime::UNIX_EPOCH;

    println!("Starting to capture power telemetry");
    while let Ok(PowerUsage { date, kwh}) = rx.recv() {
        // Update prices.

        println!("date: {:?}, kwh: {:?}", date, kwh);
        if last_power_reading == 0.0 {
            // Use the first reading as a baseline.
            last_power_reading = kwh;
            last_date = date;
            continue;
        }
        let usage = kwh - last_power_reading;
        if usage < 0.0 {
            panic!("Usage should never drop!");
        }

        let data_value: PowerUsageCsvFormat = if usage > 0.0 {
            // Find the price that matches this period
            let usage_cost: f32 = match octopus.get_price_for_period(last_date, date).await {
                Ok((matched_cost, Some(second_cost))) => {
                    // We fall inside a second bucket, so fetch that price too.
                    // Calculate the delta between the two timestamps
                    let total_delta = (date-last_date).num_seconds() as f32;
                    let mult_a = (matched_cost.to - last_date).num_seconds() as f32 / total_delta;
                    let mult_b = (date - second_cost.from).num_seconds() as f32  / total_delta;
                    // And thus determine how much power was used (approx) in each period.
                    (matched_cost.cost * (usage * mult_a)) + second_cost.cost * (usage * mult_b)
                },
                Ok((matched_cost, None)) => {
                    // Otherwise, straightforward to calculate.
                    matched_cost.cost * usage
                }
                Err(e) => {
                    panic!("Failure to handle cost at {:?}. No applicable cost found: {:}", date, e)
                },
            };
            println!("Calculated {:?} for {:?} ({:?} kwh)", usage_cost, date, kwh);
            PowerUsageCsvFormat { date, usage, total_usage: kwh, cost: usage_cost }
        } else {
           PowerUsageCsvFormat { date, usage, total_usage: kwh, cost: 0.0 }
        };
        if let Err(db_err) = database.clone().insert_new_data_value(&data_value).await {
            println!("Failed to insert data into DB: {:?}", db_err);
        }
        if let Err(csv_err) = f.write(&data_value.to_string().as_bytes()) {
            println!("Failed to insert data into the CSV: {:?}", csv_err);
        }
        last_power_reading = kwh;
        last_date = date;
    }
    Ok(())
}
