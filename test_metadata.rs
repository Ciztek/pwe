use std::path::Path;

fn main() {
    let path = Path::new(r"d:\projects\pwe\library\Just Communication.mp3");

    println!("Testing metadata extraction on: {}", path.display());

    match pwe_karaoke::audio::metadata::extract_basic_metadata(path) {
        Ok(metadata) => {
            println!("\n=== BASIC METADATA ===");
            println!("Title: {:?}", metadata.title);
            println!("Artist: {:?}", metadata.artist);
            println!("Album: {:?}", metadata.album);
            println!("Duration: {:?} seconds", metadata.duration_secs);
        },
        Err(e) => println!("Error extracting basic metadata: {}", e),
    }

    println!("\n\n=== FULL METADATA ===");
    match pwe_karaoke::audio::metadata::extract_metadata(path) {
        Ok(metadata) => {
            println!("Title: {:?}", metadata.title);
            println!("Artist: {:?}", metadata.artist);
            println!("Album: {:?}", metadata.album);
            println!("Album Artist: {:?}", metadata.album_artist);
            println!("Track Number: {:?}", metadata.track_number);
            println!("Genre: {:?}", metadata.genre);
            println!("Date: {:?}", metadata.date);
            println!("Duration: {:?} seconds", metadata.duration_secs);
            println!("Has Lyrics: {}", metadata.lyrics.is_some());
            println!(
                "Has Cover Art: {} (size: {} bytes)",
                metadata.cover_art.is_some(),
                metadata.cover_art.as_ref().map(|v| v.len()).unwrap_or(0)
            );
        },
        Err(e) => println!("Error extracting full metadata: {}", e),
    }
}
