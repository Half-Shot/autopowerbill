use chrono::{DateTime, Utc};


pub struct PowerUsageCsvFormat {
    pub date: DateTime<Utc>,
    pub usage: f32,
    pub total_usage: f32,
    pub cost: f32,
}

impl std::fmt::Display for PowerUsageCsvFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      writeln!(f, "{:},{:},{:},{:}", self.date.to_rfc3339(), self.usage, self.total_usage, self.cost)
    }
}