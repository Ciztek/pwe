use crate::lrc::error::LrcError;
use crate::lrc::timestamp::TimeStamp;
use crate::lrc::tokenizer;
use crate::lrc::tokens::{LrcEvent, LyricSegment, Token};

/// Parse a vector of tokens (one line) into an LrcEvent.
pub fn parse_tokens(tokens: Vec<Token>) -> Result<Option<LrcEvent>, LrcError> {
    if tokens.is_empty() {
        return Ok(None);
    }

    let mut it = tokens.into_iter().peekable();

    // Metadata (only allowed as first token in the line)
    if let Some(Token::Metadata { key, value }) = it.peek().cloned() {
        return Ok(Some(LrcEvent::Metadata { key, value }));
    }

    let mut timestamps: Vec<TimeStamp> = Vec::new();
    while let Some(Token::Timestamp(ts)) = it.peek().cloned() {
        timestamps.push(ts);
        it.next();
    }

    if timestamps.is_empty() {
        return Ok(None);
    }

    let mut segments: Vec<LyricSegment> = Vec::new();
    let mut buffer = String::new();

    for tok in it {
        match tok {
            Token::EnhancedTimestamp(ts) => {
                if !buffer.is_empty() {
                    segments.push(LyricSegment {
                        ts: None,
                        text: buffer.clone(),
                    });
                    buffer.clear();
                }
                segments.push(LyricSegment {
                    ts: Some(ts),
                    text: String::new(),
                });
            },
            Token::Text(s) => {
                if let Some(last) = segments.last_mut() {
                    if last.ts.is_some() && last.text.is_empty() {
                        last.text.push_str(&s);
                        continue;
                    }
                }
                buffer.push_str(&s);
            },
            // tok => Err(LrcError::UnexpectedToken(tok)),
            _ => {},
        }
    }
    if !buffer.is_empty() {
        segments.push(LyricSegment {
            ts: None,
            text: buffer,
        });
    }

    Ok(Some(LrcEvent::Lyric {
        timestamps,
        segments,
    }))
}

/// Parse an entire LRC text into events.
pub fn parse_lrc(text: &str) -> Result<Vec<LrcEvent>, LrcError> {
    let mut out = Vec::new();
    let token_lines = tokenizer::tokenize(text)?;

    for toks in token_lines {
        if let Some(ev) = parse_tokens(toks)? {
            out.push(ev);
        }
    }

    Ok(out)
}
