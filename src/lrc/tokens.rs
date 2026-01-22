use crate::lrc::timestamp::TimeStamp;

#[derive(Debug, Clone)]
pub enum Token {
    Metadata { key: String, value: String },
    Timestamp(TimeStamp),
    EnhancedTimestamp(TimeStamp),
    Text(String),
}

#[derive(Debug, Clone)]
pub enum LrcEvent {
    #[allow(dead_code)]
    Metadata { key: String, value: String },
    Lyric {
        timestamps: Vec<TimeStamp>,
        segments: Vec<LyricSegment>,
    },
}

#[derive(Debug, Clone)]
pub struct LyricSegment {
    pub ts: Option<TimeStamp>,
    pub text: String,
}
