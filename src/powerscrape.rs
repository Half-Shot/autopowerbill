use chrono::{DateTime, Utc};
use rumqttc::{Client, MqttOptions, QoS};
use rumqttc::Event::{Incoming, Outgoing};
use rumqttc::v4::Publish;
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Deserialize)]
struct PowerStatusPacket {
    // We assume here
    // #[serde(rename = "Time")]
    // time: String,
    #[serde(rename = "ENERGY")]
    energy: PowerStatus,
}

#[derive(Deserialize)]
struct PowerStatus {
    // #[serde(rename = "ApparentPower")]
    // apparent_power: u16,
    // #[serde(rename = "Current")]
    // current: f32,
    // #[serde(rename = "Factor")]
    // factor: f32,
    // #[serde(rename = "Period")]
    // period: u16,
    // #[serde(rename = "Power")]
    // power: u16,
    // #[serde(rename = "Today")]
    // today: f32,
    #[serde(rename = "Total")]
    total: f32,
    // #[serde(rename = "TotalStartTime")]
    // total_start_time: String,
    // #[serde(rename = "Voltage")]
    // voltage: u16,
    // #[serde(rename = "Yesterday")]
    // yesterday: f32,
}

pub struct PowerUsage {
    pub date: DateTime<Utc>,
    pub kwh: f32,
}

pub fn start_power_scrape_thread(tx: Sender<PowerUsage>) -> JoinHandle<()> {
    let mut mqttoptions = MqttOptions::new("rumqtt-async", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    let (client, mut eventloop) = Client::new(mqttoptions, 10);
    client.subscribe("tele/tasmota_D5E4EC/SENSOR", QoS::AtMostOnce).unwrap();
    
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./powerusage.csv")
        .unwrap();

    return thread::spawn(move || {
        for notification in eventloop.iter() {
            match notification {
                Ok(Incoming(evt)) => {
                    if let rumqttc::Packet::Publish(Publish { payload, .. }) = evt {
                        let status: PowerStatusPacket = serde_json::from_slice(&payload).unwrap();
                        let now = chrono::Utc::now();
                        f.write(format!("{:},{:}\n", now.to_rfc3339(), status.energy.total).as_bytes()).unwrap();
                        tx.send(PowerUsage { kwh: status.energy.total, date: now }).unwrap();
                    }
                } 
                // We don't care about outgoing
                Ok(Outgoing(..)) => (),
                Err(e) => {
                    println!("mqtt connection error {:?}", e);
                }
            }
        }
    });

}
