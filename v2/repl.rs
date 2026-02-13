use crate::library::{self, Library, Playlist};
use crate::song::Song;
use std::io::{self, Write};
use std::str::SplitWhitespace;

fn next_arg<'a>(args: &mut SplitWhitespace<'a>, usage: &str) -> Option<&'a str> {
    let Some(arg) = args.next() else {
        println!("{}", usage);
        return None;
    };
    Some(arg)
}

fn parse_index(idx_str: &str) -> Option<usize> {
    let Ok(idx) = idx_str.parse::<usize>() else {
        println!("Index must be a number");
        return None;
    };
    Some(idx)
}

fn get_song(library: &Library, idx: usize) -> Option<Song> {
    let Some(song) = library.songs().get(idx).cloned() else {
        println!("Invalid song index");
        return None;
    };
    Some(song)
}

fn list(library: &mut Library, args: &mut SplitWhitespace) -> bool {
    match args.next() {
        Some("all") => {
            for (i, song) in library.songs().iter().enumerate() {
                println!("{}: {}", i, song.path().display());
            }
            return true;
        },
        Some("playlists") => {
            println!(
                "{}",
                serde_json::to_string_pretty(&library.playlists()).unwrap()
            );
            return true;
        },
        Some("playlist") => {
            let Some(name) = next_arg(args, "Usage: list playlist <name>") else {
                return true;
            };

            let Some(pl) = library.playlists().iter().find(|p| p.name == name) else {
                println!("Playlist '{}' not found", name);
                return true;
            };

            for (i, song) in pl.entries.iter().enumerate() {
                println!("{}: {}", i, song.path().display());
            }
            return true;
        },
        _ => {
            println!("Usage: list all | playlists | playlist <name>");
            return true;
        },
    }
}

fn create(library: &mut Library, args: &mut SplitWhitespace) -> bool {
    let Some(name) = next_arg(args, "Usage: create <playlist_name>") else {
        return true;
    };

    library.playlist_create(name);
    println!("Playlist '{}' created", name);
    true
}

fn delete(library: &mut Library, args: &mut SplitWhitespace) -> bool {
    let Some(name) = next_arg(args, "Usage: delete <playlist_name>") else {
        return true;
    };

    library.playlist_delete(name);
    println!("Playlist '{}' deleted", name);
    true
}

fn add(library: &mut Library, args: &mut SplitWhitespace) -> bool {
    let Some(pl_name) = next_arg(args, "Usage: add <playlist> <song_idx>") else {
        return true;
    };

    let Some(idx_str) = next_arg(args, "Usage: add <playlist> <song_idx>") else {
        return true;
    };

    let Some(idx) = parse_index(idx_str) else {
        return true;
    };

    let Some(song) = get_song(library, idx) else {
        return true;
    };

    library.playlist_add_song(pl_name, &song);
    println!(
        "Added '{}' to playlist '{}'",
        song.path().display(),
        pl_name
    );
    true
}

fn remove(library: &mut Library, args: &mut SplitWhitespace) -> bool {
    let Some(pl_name) = next_arg(args, "Usage: remove <playlist> <local_song_idx>") else {
        return true;
    };

    let Some(idx_str) = next_arg(args, "Usage: remove <playlist> <local_song_idx>") else {
        return true;
    };

    let Some(idx) = parse_index(idx_str) else {
        return true;
    };

    let Some(pl) = library.playlists().iter().find(|p| p.name == pl_name) else {
        println!("Playlist '{}' not found", pl_name);
        return true;
    };

    let Some(song) = pl.entries.get(idx).cloned() else {
        println!("Invalid local song index");
        return true;
    };

    library.playlist_remove_song(pl_name, &song);
    println!(
        "Removed '{}' from playlist '{}'",
        song.path().display(),
        pl_name
    );
    true
}

fn exit(_: &mut Library, _: &mut SplitWhitespace) -> bool {
    println!("Exiting REPL...");
    false
}

fn command_unknown(_: &mut Library, parts: &mut SplitWhitespace) -> bool {
    println!(
        "Unknown command: '{}', type 'help' for commands",
        parts.collect::<Vec<_>>().join(" ")
    );
    true
}

fn help(_: &mut Library, _: &mut SplitWhitespace) -> bool {
    println!("Available commands:");
    println!("  help                                Show this message");
    println!("  exit                                Exit the REPL");
    println!("  list all                            List all songs with index");
    println!("  list playlists                      List all playlists (JSON)");
    println!("  list playlist <name>                List songs in playlist with index");
    println!("  create <playlist_name>              Create a new playlist");
    println!("  delete <playlist_name>              Delete a playlist");
    println!("  add <playlist> <global_song_idx>    Add song to playlist");
    println!("  remove <playlist> <local_song_idx>  Remove song from playlist");
    true
}

pub fn run_repl(library: &mut Library) {
    println!("Debug REPL for Library");
    println!("Type 'help' for commands");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input");
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

        let handler: fn(&mut Library, &mut SplitWhitespace) -> bool = match cmd {
            "help" => help,
            "exit" => exit,
            "list" => list,
            "create" => create,
            "delete" => delete,
            "add" => add,
            "remove" => remove,
            _ => command_unknown,
        };

        if !handler(library, &mut parts) {
            break;
        }
    }

    // Auto-save on exit
    if let Err(e) = library.save() {
        println!("Failed to save library: {:?}", e);
    }
}
