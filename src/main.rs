mod octopus;
mod powerscrape;

use chrono::{DateTime, Utc};
use octopus::get_agile_prices;
use powerscrape::{start_power_scrape_thread, PowerUsage};
use std::{env, fs::OpenOptions, io::Write};
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
    let (tx, rx) = mpsc::channel::<PowerUsage>();
    start_power_scrape_thread(tx);
    let mut agile_costs = get_agile_prices().await?;
    let usage_cost = env::args().collect::<Vec<String>>().get(1).unwrap_or(&"0".to_string()).parse::<f32>().unwrap();
    let mut running_total_pence = usage_cost;
    let mut last_power_reading: f32 = 0.0;
    let mut last_date: DateTime<Utc> = DateTime::UNIX_EPOCH;

    let mut f = OpenOptions::new()
    .create(true)
    .append(true)
    .open("./costfile.csv")
    .unwrap();

    let mut last_octoput_fetch = Utc::now();

    while let Ok(PowerUsage { date, kwh}) = rx.recv() {
        // Update prices.
        if (Utc::now() - last_octoput_fetch).num_hours() > 3 {
            agile_costs = get_agile_prices().await?;
            last_octoput_fetch = Utc::now();
        }

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
            if let Some(matched_cost) = agile_costs.iter().find(|c| c.from <= last_date && c.to >= last_date) {
                // We might fall inside a second bucket, so fetch that price too.
                let usage_cost = if let Some(second_cost) = agile_costs.iter().find(|c| c.from <= date && c.to >= date && c.from != matched_cost.from) {
                    // Calculate the delta between the two timestamps
                    let total_delta = (date-last_date).num_seconds() as f32;
                    let mult_a = (matched_cost.to - last_date).num_seconds() as f32 / total_delta;
                    let mult_b = (date - second_cost.from).num_seconds() as f32  / total_delta;
                    // And thus determine how much power was used (approx) in each period.
                    (matched_cost.cost * (usage * mult_a)) + second_cost.cost * (usage * mult_b)
                } else {
                    // Otherwise, straightforward to calculate.
                    matched_cost.cost * usage
                };
                running_total_pence += usage_cost;
                println!("Calculated {:?} for {:?} ({:?} kwh)", usage_cost, date, kwh);
                f.write(PowerUsageCsvFormat { date, usage, total_usage: kwh, cost: usage_cost }.to_string().as_bytes()).unwrap();
            } else {
                println!("Failure to handle cost at {:?}. No applicable cost found.", date);
            }
        } else {
            f.write(PowerUsageCsvFormat { date, usage, total_usage: kwh, cost: 0.0 }.to_string().as_bytes()).unwrap();
        }

        last_power_reading = kwh;
        last_date = date;
    }
    println!("Calculated cost: £{:.2}", running_total_pence.ceil() / 100.0);
    Ok(())
}
