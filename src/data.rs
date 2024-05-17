use chrono::{DateTime, Utc};


pub struct PowerUsageCsvFormat {
    pub date: DateTime<Utc>,
    pub usage: f32,
    pub total_usage: f32,
    pub cost: f32,
}

impl ToString for PowerUsageCsvFormat {
    fn to_string(&self) -> String {
        format!("{:},{:},{:},{:}\n", self.date.to_rfc3339(), self.usage, self.total_usage, self.cost).to_string()
    }
}