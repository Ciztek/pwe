use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};

use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{MetadataOptions, MetadataRevision, StandardTagKey, StandardVisualKey},
    probe::Hint,
};

#[derive(Debug, Clone)]
pub struct Visual {
    pub data: Vec<u8>,
    pub mime: String,
    pub usage: Option<StandardVisualKey>,
}

#[derive(Clone, Default)]
pub struct AudioMetadata {
    pub standard_tags: HashMap<StandardTagKey, Vec<String>>,
    pub raw_tags: HashMap<String, Vec<String>>,
    pub visuals: Vec<Visual>,
    pub duration_secs: Option<u64>,
}

impl Debug for AudioMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioMetadata")
            .field("standard_tags", &self.standard_tags)
            .field("raw_tags", &self.raw_tags)
            .field("visuals_count", &self.visuals.len())
            .field("duration_secs", &self.duration_secs)
            .finish()
    }
}

#[derive(Default)]
struct MetadataSink {
    out: AudioMetadata,
}

impl MetadataSink {
    fn absorb_revision(&mut self, rev: &MetadataRevision) {
        self.absorb_tags(rev);
        self.absorb_visuals(rev);
    }

    fn absorb_tags(&mut self, rev: &MetadataRevision) {
        for tag in rev.tags() {
            let value = tag.value.to_string();

            if let Some(key) = tag.std_key {
                self.out.standard_tags.entry(key).or_default().push(value);
            } else {
                self.out
                    .raw_tags
                    .entry(tag.key.clone())
                    .or_default()
                    .push(value);
            }
        }
    }

    fn absorb_visuals(&mut self, rev: &MetadataRevision) {
        for visual in rev.visuals() {
            self.out.visuals.push(Visual {
                data: visual.data.to_vec(),
                mime: visual.media_type.clone(),
                usage: visual.usage,
            });
        }
    }

    fn finish(self) -> AudioMetadata {
        self.out
    }
}

pub fn extract_metadata(path: impl AsRef<Path>) -> Result<AudioMetadata> {
    let path = path.as_ref();

    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let mut probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut sink = MetadataSink::default();

    if let Some(metadata) = probed.metadata.get() {
        if let Some(rev) = metadata.current() {
            sink.absorb_revision(rev);
        }
    }

    if let Some(rev) = probed.format.metadata().current() {
        sink.absorb_revision(rev);
    }

    if let Some(track) = probed.format.default_track() {
        if let (Some(tb), Some(frames)) =
            (track.codec_params.time_base, track.codec_params.n_frames)
        {
            sink.out.duration_secs = Some(tb.calc_time(frames).seconds);
        }
    }

    Ok(sink.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use symphonia::core::meta::Visual as SymVisual;
    use symphonia::core::meta::{MetadataBuilder, Tag, Value};

    #[test]
    fn collects_standard_and_raw_tags() {
        let mut rev_builder = MetadataBuilder::new();

        rev_builder.add_tag(Tag::new(
            Some(StandardTagKey::TrackTitle),
            "TITLE",
            Value::String("Test Song".into()),
        ));

        rev_builder.add_tag(Tag::new(None, "TXXX:FOO", Value::String("Bar".into())));

        let mut sink = MetadataSink::default();
        let rev = rev_builder.metadata();
        sink.absorb_revision(&rev);
        let meta = sink.finish();

        assert_eq!(
            meta.standard_tags.get(&StandardTagKey::TrackTitle).unwrap(),
            &vec!["Test Song".to_string()]
        );

        assert_eq!(
            meta.raw_tags.get("TXXX:FOO").unwrap(),
            &vec!["Bar".to_string()]
        );
    }

    #[test]
    fn collects_visuals() {
        let mut rev_builder = MetadataBuilder::new();

        rev_builder.add_visual(SymVisual {
            data: vec![1, 2, 3].into_boxed_slice(),
            media_type: "image/jpeg".into(),
            dimensions: None,
            bits_per_pixel: None,
            color_mode: None,
            usage: Some(StandardVisualKey::FrontCover),
            tags: vec![],
        });

        let mut sink = MetadataSink::default();
        let rev = rev_builder.metadata();
        sink.absorb_revision(&rev);
        let meta = sink.finish();

        assert_eq!(meta.visuals.len(), 1);
        assert_eq!(meta.visuals[0].data, vec![1, 2, 3]);
        assert_eq!(meta.visuals[0].mime, "image/jpeg");
    }
}
