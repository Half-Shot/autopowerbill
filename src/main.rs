mod octopus;
mod powerscrape;
mod cli;
use chrono::{DateTime, Utc};
use powerscrape::{start_power_scrape_thread, PowerUsage};
use std::{fs::OpenOptions, io::Write};
use std::sync::mpsc;

struct PowerUsageCsvFormat {
    date: DateTime<Utc>,
    usage: f32,
    total_usage: f32,
    cost: f32,
}

impl ToString for PowerUsageCsvFormat {
    fn to_string(&self) -> String {
        format!("{:},{:},{:},{:}", self.date.to_rfc3339(), self.usage, self.total_usage, self.cost).to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let args = cli::get_args();    
    let (tx, rx) = mpsc::channel::<PowerUsage>();

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

        if usage > 0.0 {
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
            f.write(PowerUsageCsvFormat { date, usage, total_usage: kwh, cost: usage_cost }.to_string().as_bytes()).unwrap();
        } else {
            f.write(PowerUsageCsvFormat { date, usage, total_usage: kwh, cost: 0.0 }.to_string().as_bytes()).unwrap();
        }

        last_power_reading = kwh;
        last_date = date;
    }
    Ok(())
}
