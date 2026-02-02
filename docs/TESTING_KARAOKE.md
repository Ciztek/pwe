# Testing Karaoke Features

## Quick Test

1. **Place a test audio file and LRC file together:**

   ```txt
   test_song.mp3
   test_song.lrc    (must have the same name!)
   ```

2. **Add the song to your library** via the Library view

3. **Play the song** - lyrics will automatically load

4. **Switch to Karaoke view** to see synchronized lyrics

## LRC File Format

LRC files are simple text files with timestamps:

```lrc
[ti:Song Title]
[ar:Artist Name]
[al:Album Name]

[00:00.00]First line of lyrics
[00:05.50]Second line appears at 5.5 seconds
[00:12.30]Third line at 12.3 seconds
```

## Creating Your Own LRC Files

### Format

- Timestamps: `[MM:SS.XX]` where MM=minutes, SS=seconds, XX=centiseconds
- Each line: `[timestamp]Lyric text`
- Metadata (optional): `[ti:], [ar:], [al:], [by:]`

### Example

```lrc
[ti:My Song]
[ar:My Artist]

[00:00.00]Welcome to karaoke
[00:03.50]This is so much fun
[00:07.20]Sing along with me
[00:11.00]Let's rock this song!
```

## Testing the Provided Test File

A `test_song.lrc` file has been created in the project root. To test:

1. Find any audio file (MP3, WAV, FLAC, etc.)
2. Rename it to `test_song.mp3` (or `.wav`, `.flac`, etc.)
3. Place both files in the same directory
4. Add to library and play

## Troubleshooting

### "No lyrics file found"

- Make sure the LRC file has **exactly** the same name as the audio file
- Example: `my_song.mp3` needs `my_song.lrc` (not `my_song.txt` or `MySong.lrc`)

### "Failed to parse lyrics"

- Check the LRC file format
- Timestamps must be in `[MM:SS.XX]` format
- Make sure there are no special characters breaking the format

### Lyrics don't sync properly

- Verify timestamp format in LRC file
- Check if timestamps are in chronological order
- Test with the provided `test_song.lrc` which has correct formatting

### Lyrics appear but don't highlight

- Check if the audio is actually playing (not paused)
- Look at the bottom playback bar to verify time is progressing
- Check console logs for errors (run with `RUST_LOG=info cargo run`)

## Where to Find LRC Files

Many karaoke and lyrics websites provide LRC files:

- Search for "[song name] lrc file" online
- Some music players can create/export LRC files
- You can create them manually using a text editor

## Features

✅ **Auto-detection**: LRC files are automatically found when you play a song
✅ **Color coding**: Current line is highlighted, upcoming lines are shown
✅ **Smooth scrolling**: Lyrics scroll as the song progresses
✅ **Error messages**: Clear feedback if LRC file is missing or invalid
✅ **Multi-format**: Works with MP3, WAV, FLAC, OGG, M4A, AAC

## Console Logging

Run with logging to debug:

```bash
# Windows PowerShell
$env:RUST_LOG="info"; cargo run

# Linux/Mac
RUST_LOG=info cargo run
```

Look for these log messages:

- `Looking for LRC file at: ...`
- `Found LRC file, parsing...`
- `Loaded X lyric lines`
