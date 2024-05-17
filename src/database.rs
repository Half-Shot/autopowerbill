use std::str::FromStr;

use rust_decimal::{prelude::FromPrimitive, Decimal};
use tokio_postgres::{Config, Error};
use crate::data::PowerUsageCsvFormat;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
// Need a single table for power usage
// 

#[derive(Clone)]
pub struct Database {
    cfg: Config,
}

impl Database {
    pub fn new(database_config: String) -> Result<Self, Error> {
        let cfg: Config = Config::from_str(database_config.as_str())?;
        Ok(Database { cfg })
    }

    pub async fn ensure_schema(self) -> Result<(), Error> {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_verify(SslVerifyMode::NONE);
        let connector = MakeTlsConnector::new(builder.build());
        
        let (client, connection) = self.cfg.connect(connector).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        println!("connected es");
        client.batch_execute("
            CREATE TABLE IF NOT EXISTS power_usage2 (
                date          TIMESTAMP WITHOUT TIME ZONE PRIMARY KEY,
                usage         NUMERIC NOT NULL,
                total_usage   NUMERIC NOT NULL,
                cost          NUMERIC NOT NULL
            );
        ").await?;
        println!("executed batch execute");
        Ok(())
    }
    
    pub async fn insert_new_data_value(self, data: &PowerUsageCsvFormat) -> Result<(), Error>  {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_verify(SslVerifyMode::NONE);
        let connector = MakeTlsConnector::new(builder.build());
        let (client, connection) = self.cfg.connect(connector).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        client.execute(
            "INSERT INTO power_usage VALUES ($1,$2,$3,$4);",
            &[
                &data.date.naive_utc(),
                &Decimal::from_f32(data.usage).unwrap(),
                &Decimal::from_f32(data.total_usage).unwrap(),
                &Decimal::from_f32(data.cost).unwrap()
            ],
        ).await?;
        Ok(())
    }
}

