use regex::Captures;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeStamp {
    pub min: u32,
    pub sec: u32,
    pub ms: u32,
}

impl TimeStamp {
    #[allow(clippy::cast_lossless)]
    pub const fn to_millis(self) -> u64 {
        ((self.min as u64) * 60_000) + ((self.sec as u64) * 1_000) + (self.ms as u64)
    }

    pub fn from_captures(cap: &Captures) -> Option<Self> {
        let min = cap.name("min")?.as_str().parse::<u32>().ok()?;
        let sec = cap.name("sec")?.as_str().parse::<u32>().ok()?;
        let ms_raw = cap
            .name("ms")
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .unwrap_or(0);

        let ms = cap.name("ms").map_or(0, |m| {
            let s = m.as_str();
            match s.len() {
                1 => ms_raw * 100,
                2 => ms_raw * 10,
                _ => ms_raw,
            }
        });

        Some(Self { min, sec, ms })
    }
}
