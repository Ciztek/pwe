use crate::lrc::error::LrcError;
use crate::lrc::timestamp::TimeStamp;
use crate::lrc::tokens::Token;
use regex::Regex;
use std::sync::OnceLock;

static META_RE: OnceLock<Regex> = OnceLock::new();
static TS_RE: OnceLock<Regex> = OnceLock::new();
static ENH_RE: OnceLock<Regex> = OnceLock::new();

fn get_meta_re() -> &'static Regex {
    META_RE.get_or_init(|| {
        #[allow(clippy::expect_used)]
        Regex::new(r"^\[(?P<key>[A-Za-z]+):\s+(?P<value>.*)\]$")
            .expect("Failed to compile metadata regex")
    })
}

fn get_ts_re() -> &'static Regex {
    TS_RE.get_or_init(|| {
        #[allow(clippy::expect_used)]
        Regex::new(r"\[(?P<min>\d{1,2}):(?P<sec>\d{1,2})(?:\.(?P<ms>\d{1,3}))?\]")
            .expect("Failed to compile timestamp regex")
    })
}

fn get_enh_re() -> &'static Regex {
    ENH_RE.get_or_init(|| {
        #[allow(clippy::expect_used)]
        Regex::new(r"<(?P<min>\d{1,2}):(?P<sec>\d{1,2})(?:\.(?P<ms>\d{1,3}))?>")
            .expect("Failed to compile enhanced timestamp regex")
    })
}

/// Tokenize a single line into tokens.
pub fn tokenize_line(line: &str) -> Result<Vec<Token>, LrcError> {
    let meta_re = get_meta_re();
    let ts_re = get_ts_re();
    let enh_re = get_enh_re();
    let trimmed = line.trim();

    if let Some(cap) = meta_re.captures(trimmed) {
        // If the regex matched, these named captures must exist
        let key = cap
            .name("key")
            .map_or_else(String::new, |m| m.as_str().to_string());
        let value = cap
            .name("value")
            .map_or_else(String::new, |m| m.as_str().to_string());
        return Ok(vec![Token::Metadata { key, value }]);
    }

    let cursor = trimmed;
    let mut out: Vec<Token> = Vec::new();

    let mut matches: Vec<(usize, usize, bool)> = ts_re
        .find_iter(cursor)
        .map(|m| (m.start(), m.end(), true))
        .collect();

    matches.extend(
        enh_re
            .find_iter(cursor)
            .map(|m| (m.start(), m.end(), false)),
    );

    // sort by start position
    matches.sort_by_key(|m| m.0);

    let mut last = 0usize;

    for (start, end, is_normal) in matches {
        if start > last {
            let text = &cursor[last..start];
            if !text.is_empty() {
                out.push(Token::Text(text.to_string()));
            }
        }

        let slice = &cursor[start..end];
        if is_normal {
            if let Some(cap) = ts_re.captures(slice) {
                if let Some(ts) = TimeStamp::from_captures(&cap) {
                    out.push(Token::Timestamp(ts));
                } else {
                    return Err(LrcError::InvalidTimestamp(slice.to_string()));
                }
            }
        } else if let Some(cap) = enh_re.captures(slice) {
            if let Some(ts) = TimeStamp::from_captures(&cap) {
                out.push(Token::EnhancedTimestamp(ts));
            } else {
                return Err(LrcError::InvalidTimestamp(slice.to_string()));
            }
        }

        last = end;
    }

    if last < cursor.len() {
        let text = &cursor[last..];
        if !text.is_empty() {
            out.push(Token::Text(text.to_string()));
        }
    }

    Ok(out)
}

pub fn tokenize(text: &str) -> Result<Vec<Vec<Token>>, LrcError> {
    let mut res = Vec::new();
    for line in text.lines() {
        let toks = tokenize_line(line)?;
        res.push(toks);
    }
    Ok(res)
}
