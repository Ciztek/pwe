use regex::Captures;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeStamp {
    pub min: u32,
    pub sec: u32,
    pub ms: u32,
}

impl TimeStamp {
    pub fn to_millis(&self) -> u64 {
        ((self.min as u64) * 60_000) + ((self.sec as u64) * 1_000) + (self.ms as u64)
    }

    pub fn from_captures(cap: &Captures) -> Option<Self> {
        let min = cap.name("min")?.as_str().parse::<u32>().ok()?;
        let sec = cap.name("sec")?.as_str().parse::<u32>().ok()?;
        let ms = cap
            .name("ms")
            .map(|m| m.as_str().parse::<u32>().ok())
            .flatten()
            .unwrap_or(0);

        let ms = match cap.name("ms") {
            Some(m) => {
                let s = m.as_str();
                match s.len() {
                    1 => ms * 100,
                    2 => ms * 10,
                    _ => ms,
                }
            },
            None => 0,
        };

        Some(TimeStamp { min, sec, ms })
    }
}
