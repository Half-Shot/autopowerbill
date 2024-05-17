use clap::Parser;

/// Monitors telemetry from a Tasmota Smart Plug and
/// accurate records the costs for each period of use,
/// using Octopus Agile energy prices.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// MQTT broker host name
    #[arg(long)]
    pub host: String,
    /// MQTT broker port
    #[arg(short, long)]
    pub port: Option<u16>,
    /// MQTT topic
    #[arg(short, long)]
    pub topic: String,
    /// Octopus Energy product code
    #[arg(long)]
    pub product_code: String,
    /// Octopus Energy tariff code
    #[arg(long)]
    pub tariff_code: String,
    /// Postgres database connection string
    #[arg(long)]
    pub database: String,
}

pub fn get_args() -> Args {
    Args::parse()
}