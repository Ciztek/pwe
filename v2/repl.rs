use crate::library::{Library, Playlist};
use crate::song::Song;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, SerializeStruct, Serializer};
use std::io::{self, Write};
use std::str::SplitWhitespace;

enum ReplControl {
    Continue,
    Exit,
}

type ReplResult = Result<ReplControl, String>;
type CommandHandler = fn(&mut Library, &mut SplitWhitespace) -> ReplResult;

fn next_arg<'a>(args: &mut SplitWhitespace<'a>, usage: &str) -> Result<&'a str, String> {
    let Some(arg) = args.next() else {
        return Err(usage.to_string());
    };
    Ok(arg)
}

fn parse_index(idx_str: &str) -> Result<usize, String> {
    let Ok(idx) = idx_str.parse::<usize>() else {
        return Err("Index must be a number".to_string());
    };
    Ok(idx)
}

fn get_song(library: &Library, idx: usize) -> Result<Song, String> {
    let Some(song) = library.songs().get(idx).cloned() else {
        return Err("Invalid song index".to_string());
    };
    Ok(song)
}

struct IndexedSong<'a> {
    index: usize,
    song: &'a Song,
}

impl<'a> Serialize for IndexedSong<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("IndexedSong", 2)?;
        state.serialize_field("index", &self.index)?;
        state.serialize_field("song", &self.song)?;
        state.end()
    }
}

struct IndexedSongsView<'a> {
    songs: &'a [Song],
}

impl<'a> Serialize for IndexedSongsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.songs.len()))?;
        for (index, song) in self.songs.iter().enumerate() {
            seq.serialize_element(&IndexedSong { index, song })?;
        }
        seq.end()
    }
}

struct PlaylistEntriesView<'a> {
    entries: &'a [Song],
}

impl<'a> Serialize for PlaylistEntriesView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.entries.len()))?;
        for (index, song) in self.entries.iter().enumerate() {
            seq.serialize_element(&IndexedSong { index, song })?;
        }
        seq.end()
    }
}

struct PlaylistView<'a> {
    playlist: &'a Playlist,
}

impl<'a> Serialize for PlaylistView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let pl = self.playlist;
        let mut state = serializer.serialize_struct("Playlist", 2)?;
        state.serialize_field("name", &pl.name)?;
        state.serialize_field(
            "entries",
            &PlaylistEntriesView {
                entries: &pl.entries,
            },
        )?;
        state.end()
    }
}

struct PlaylistsView<'a> {
    playlists: &'a [Playlist],
}

impl<'a> Serialize for PlaylistsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.playlists.len()))?;
        for playlist in self.playlists {
            seq.serialize_element(&PlaylistView { playlist })?;
        }
        seq.end()
    }
}

struct LibraryAllView<'a> {
    library: &'a Library,
}

impl<'a> Serialize for LibraryAllView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(
            "songs",
            &IndexedSongsView {
                songs: self.library.songs(),
            },
        )?;
        map.serialize_entry(
            "playlists",
            &PlaylistsView {
                playlists: self.library.playlists(),
            },
        )?;
        map.end()
    }
}

fn list(library: &mut Library, args: &mut SplitWhitespace) -> ReplResult {
    let Some(sub) = args.next() else {
        return Err("Usage: list all | songs | playlists | playlist <name>".to_string());
    };

    match sub {
        "all" => {
            let view = LibraryAllView { library };
            let json = serde_json::to_string_pretty(&view)
                .map_err(|e| format!("Serialization error: {e}"))?;
            println!("{json}");
            Ok(ReplControl::Continue)
        },
        "songs" => {
            let view = IndexedSongsView {
                songs: library.songs(),
            };
            let json = serde_json::to_string_pretty(&view)
                .map_err(|e| format!("Serialization error: {e}"))?;
            println!("{json}");
            Ok(ReplControl::Continue)
        },
        "playlists" => {
            let view = PlaylistsView {
                playlists: library.playlists(),
            };
            let json = serde_json::to_string_pretty(&view)
                .map_err(|e| format!("Serialization error: {e}"))?;
            println!("{json}");
            Ok(ReplControl::Continue)
        },
        "playlist" => {
            let name = next_arg(args, "Usage: list playlist <name>")?;

            let Some(pl) = library.playlists().iter().find(|p| p.name == name) else {
                return Err(format!("Playlist '{}' not found", name));
            };

            let view = PlaylistView { playlist: pl };
            let json = serde_json::to_string_pretty(&view)
                .map_err(|e| format!("Serialization error: {e}"))?;
            println!("{json}");
            Ok(ReplControl::Continue)
        },
        _ => Err("Usage: list all | songs | playlists | playlist <name>".to_string()),
    }
}

fn create(library: &mut Library, args: &mut SplitWhitespace) -> ReplResult {
    let name = next_arg(args, "Usage: create <playlist_name>")?;
    library.playlist_create(name);
    println!("Playlist '{}' created", name);
    Ok(ReplControl::Continue)
}

fn delete(library: &mut Library, args: &mut SplitWhitespace) -> ReplResult {
    let name = next_arg(args, "Usage: delete <playlist_name>")?;
    library.playlist_delete(name);
    println!("Playlist '{}' deleted", name);
    Ok(ReplControl::Continue)
}

fn add(library: &mut Library, args: &mut SplitWhitespace) -> ReplResult {
    let pl_name = next_arg(args, "Usage: add <playlist> <song_idx>")?;
    let idx_str = next_arg(args, "Usage: add <playlist> <song_idx>")?;
    let idx = parse_index(idx_str)?;
    let song = get_song(library, idx)?;

    library.playlist_add_song(pl_name, &song);
    println!(
        "Added '{}' to playlist '{}'",
        song.path().display(),
        pl_name
    );
    Ok(ReplControl::Continue)
}

fn remove(library: &mut Library, args: &mut SplitWhitespace) -> ReplResult {
    let pl_name = next_arg(args, "Usage: remove <playlist> <playlist_idx>")?;
    let idx_str = next_arg(args, "Usage: remove <playlist> <playlist_idx>")?;
    let idx = parse_index(idx_str)?;

    let Some(pl) = library.playlists().iter().find(|p| p.name == pl_name) else {
        return Err(format!("Playlist '{}' not found", pl_name));
    };

    let Some(song) = pl.entries.get(idx).cloned() else {
        return Err("Invalid playlist index".to_string());
    };

    library.playlist_remove_song(pl_name, &song);
    println!(
        "Removed '{}' from playlist '{}'",
        song.path().display(),
        pl_name
    );
    Ok(ReplControl::Continue)
}

fn help(_: &mut Library, _: &mut SplitWhitespace) -> ReplResult {
    println!("Available commands:");
    println!("  help                             Show this message");
    println!("  exit                             Exit the REPL");
    println!("  save                             Save the library to disk");
    println!("  list all                         JSON: songs (indexed) + playlists");
    println!("  list songs                       JSON: all songs with index");
    println!("  list playlists                   JSON: all playlists with indexed entries");
    println!("  list playlist <name>             JSON: single playlist with indexed entries");
    println!("  create <playlist_name>           Create a new playlist");
    println!("  delete <playlist_name>           Delete a playlist");
    println!("  add <playlist> <song_idx>        Add song (global index) to playlist");
    println!("  remove <playlist> <playlist_idx> Remove song by playlist index");
    Ok(ReplControl::Continue)
}

fn exit(_: &mut Library, _: &mut SplitWhitespace) -> ReplResult {
    println!("Exiting REPL...");
    Ok(ReplControl::Exit)
}

fn command_unknown(_: &mut Library, parts: &mut SplitWhitespace) -> ReplResult {
    let cmd = parts.collect::<Vec<_>>().join(" ");
    Err(format!(
        "Unknown command: '{}', type 'help' for commands",
        cmd
    ))
}

fn save(library: &mut Library, _: &mut SplitWhitespace) -> ReplResult {
    match library.save() {
        Ok(_) => {
            println!("Library saved successfully");
            Ok(ReplControl::Continue)
        },
        Err(e) => Err(format!("Failed to save library: {:?}", e)),
    }
}

pub fn run_repl(library: &mut Library) {
    println!("Debug REPL for Library");
    println!("Type 'help' for commands");

    loop {
        print!("> ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {e}");
            continue;
        }

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Failed to read input: {e}");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let mut parts = input.split_whitespace();
        let Some(cmd) = parts.next() else {
            continue;
        };

        let handler: CommandHandler = match cmd {
            "help" => help,
            "exit" => exit,
            "list" => list,
            "create" => create,
            "delete" => delete,
            "add" => add,
            "remove" => remove,
            "save" => save,
            _ => command_unknown,
        };

        match handler(library, &mut parts) {
            Ok(ReplControl::Continue) => {},
            Ok(ReplControl::Exit) => break,
            Err(msg) => eprintln!("{msg}"),
        }
    }

    if let Err(e) = library.save() {
        eprintln!("Failed to save library: {:?}", e);
    }
}
