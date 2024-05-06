use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(long)]
    pub host: String,
    /// Name of the person to greet
    #[arg(short, long)]
    pub port: Option<u16>,
    /// Name of the person to greet
    #[arg(short, long)]
    pub topic: String,
    /// Name of the person to greet
    #[arg(long)]
    pub product_code: String,
    /// Name of the person to greet
    #[arg(long)]
    pub tariff_code: String,
}

pub fn get_args() -> Args {
    Args::parse()
}