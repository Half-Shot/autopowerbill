use std::fmt::Display;

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct UnitRate {
    value_exc_vat: f32,
    value_inc_vat: f32,
    valid_from: String,
    valid_to: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UnitRateResponse {
    results: Vec<UnitRate>,
}

#[derive(Debug)]
struct NoCostFoundError {
    date: DateTime<Utc>
}

impl Display for NoCostFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No cost found for timestamp {}, might indicate an error with octopus or data capture!", self.date.to_rfc3339())
    }
}

impl std::error::Error for NoCostFoundError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }


    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

#[derive(Debug)]
pub struct PowerCost {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub cost: f32,
}

pub struct Octopus {
    last_fetch: DateTime<Utc>,
    power_costs: Vec<PowerCost>,
    product_code: String,
    tariff_code: String,
}

impl Octopus {
    pub fn new(product_code: String, tariff_code: String) -> Self {
        Octopus {
            last_fetch: DateTime::UNIX_EPOCH,
            power_costs: Vec::default(),
            product_code,
            tariff_code,
        }
    }

    async fn get_agile_prices(self: &Self) -> Result<Vec<PowerCost>, Box<dyn std::error::Error>> {
        let url = format!("https://api.octopus.energy/v1/products/{:}/electricity-tariffs/{:}/standard-unit-rates/", self.product_code, self.tariff_code);
        let resp = reqwest::get(url)
        .await?
        .json::<UnitRateResponse>()
        .await?.results.into_iter().map(|f| PowerCost {
            from: f.valid_from.parse::<DateTime<Utc>>().unwrap(),
            to: f.valid_to.parse::<DateTime<Utc>>().unwrap(),
            cost: f.value_inc_vat,
        }).collect();
        Ok(resp)
    }

    pub async fn get_price_for_period(&mut self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<(&PowerCost, Option<&PowerCost>), Box<dyn std::error::Error>> {
        let now: DateTime<Utc> = Utc::now();
        // We always fetch the prices at 16, because Octopus release new prices at 16:00
        if (now - self.last_fetch).num_hours() > 3 || (now.hour() == 16 && self.last_fetch.hour() != 16) {
            self.power_costs = self.get_agile_prices().await?;
            self.last_fetch = now;
        }
        let cost1 = match self.power_costs.iter().find(|c| c.from <= start_date && c.to >= start_date) {
            Some(e) => Ok(e),
            None => Err(Box::new(NoCostFoundError { date: start_date })),
        }?;
        let cost2 = match self.power_costs.iter().find(|c| c.from <= end_date && c.to >= end_date) {
            Some(e) => Ok(e),
            None => Err(Box::new(NoCostFoundError { date: end_date })),
        }?;
        if cost1.cost == cost2.cost {
            return Ok((cost1, None));
        }
        return Ok((cost1, Some(cost2)));
    }
    
}
