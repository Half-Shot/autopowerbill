use chrono::{DateTime, Utc};
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

#[derive( Debug)]
pub struct PowerCost {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub cost: f32,
}

pub async fn get_agile_prices() -> Result<Vec<PowerCost>, Box<dyn std::error::Error>> {
    let product_code = "AGILE-24-04-03";
    let tariff_code = "E-1R-AGILE-24-04-03-D";
    let url = format!("https://api.octopus.energy/v1/products/{:}/electricity-tariffs/{:}/standard-unit-rates/", product_code, tariff_code);
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