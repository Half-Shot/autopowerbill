mod date;
use std::fs::read_to_string;
use crate::date::Date;
use std::collections::HashMap;

const USAGE_CHARGE: f32 = 0.0;

fn main() {
    println!("Hello, world!");

    let mut running_total_pence = USAGE_CHARGE;
    let costs = get_avg_costs();
    for (date, kwh) in get_power_usage() {
        running_total_pence += costs.get(&date).unwrap() * kwh;
    }

    println!("Calculated cost: £{:.2}", running_total_pence.ceil() / 100.0);
}


fn get_power_usage() -> HashMap<Date, f32> {
    read_to_string("./data/powerusage.csv").unwrap().lines().map(|line| {
        let result: Vec<&str> = line.split(',').collect();
        let date: Date = result.get(0).unwrap().parse().unwrap();
        let kwh: f32 = result.get(1).unwrap().parse().unwrap();
        (date, kwh)
    }).collect()
}

fn get_avg_costs() -> HashMap<Date, f32> {
    read_to_string("./data/avgpowercost.csv").unwrap().lines().map(|line| {
        let result: Vec<&str> = line.split(',').collect();
        let date: Date = result.get(0).unwrap().parse().unwrap();
        let pence: f32 = result.get(1).unwrap().parse().unwrap();
        (date, pence)
    }).collect()
}