use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseDateError;


#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Date {
    pub year: u8,
    pub month: u8,
    pub day: u8,
}

impl FromStr for Date {
    type Err = ParseDateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<u8> = s.splitn(3, '-').map(|f|  f.parse::<u8>().unwrap()).collect();

        Ok(Date { 
            year: *parts.get(2).unwrap(),
            month: *parts.get(1).unwrap(),
            day: *parts.get(0).unwrap()
         })
    }
}
