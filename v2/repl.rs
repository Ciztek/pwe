use crate::library::{Library, Playlist};
use crate::song::Song;
use std::io::{self, Write};

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
        let cmd = parts.next().unwrap();

        match cmd {
            "help" => {
                println!("Available commands:");
                println!("  help                     Show this message");
                println!("  exit                     Exit the REPL");
                println!("  list all                 List all songs with index");
                println!("  list playlists           List all playlists (JSON)");
                println!("  list playlist <name>     List songs in playlist with index");
                println!("  create <playlist_name>   Create a new playlist");
                println!("  delete <playlist_name>   Delete a playlist");
                println!("  add <playlist> <song_idx>    Add song to playlist");
                println!("  remove <playlist> <song_idx> Remove song from playlist");
            },

            "exit" => break,

            "list" => match parts.next() {
                Some("all") => {
                    for (i, song) in library.songs().iter().enumerate() {
                        println!("{}: {}", i, song.path().display());
                    }
                },
                Some("playlists") => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&library.playlists()).unwrap()
                    );
                },
                Some("playlist") => {
                    if let Some(name) = parts.next() {
                        if let Some(pl) = library.playlists().iter().find(|p| p.name == name) {
                            for (i, song) in pl.entries.iter().enumerate() {
                                println!("{}: {}", i, song.path().display());
                            }
                        } else {
                            println!("Playlist '{}' not found", name);
                        }
                    } else {
                        println!("Usage: list playlist <name>");
                    }
                },
                _ => println!("Usage: list all | playlists | playlist <name>"),
            },

            "create" => {
                if let Some(name) = parts.next() {
                    library.playlist_create(name);
                    println!("Playlist '{}' created", name);
                } else {
                    println!("Usage: create <playlist_name>");
                }
            },

            "delete" => {
                if let Some(name) = parts.next() {
                    library.playlist_delete(name);
                    println!("Playlist '{}' deleted", name);
                } else {
                    println!("Usage: delete <playlist_name>");
                }
            },

            "add" => {
                if let (Some(pl_name), Some(idx_str)) = (parts.next(), parts.next()) {
                    if let Ok(idx) = idx_str.parse::<usize>() {
                        if let Some(song) = library.songs().get(idx) {
                            let song = song.clone(); // clone to avoid borrow conflict
                            library.playlist_add_song(pl_name, &song);
                            println!(
                                "Added '{}' to playlist '{}'",
                                song.path().display(),
                                pl_name
                            );
                        } else {
                            println!("Invalid song index");
                        }
                    } else {
                        println!("Index must be a number");
                    }
                } else {
                    println!("Usage: add <playlist> <song_idx>");
                }
            },

            "remove" => {
                if let (Some(pl_name), Some(idx_str)) = (parts.next(), parts.next()) {
                    if let Ok(idx) = idx_str.parse::<usize>() {
                        if let Some(song) = library.songs().get(idx) {
                            let song = song.clone(); // clone to avoid borrow conflict
                            library.playlist_remove_song(pl_name, &song);
                            println!(
                                "Removed '{}' from playlist '{}'",
                                song.path().display(),
                                pl_name
                            );
                        } else {
                            println!("Invalid song index");
                        }
                    } else {
                        println!("Index must be a number");
                    }
                } else {
                    println!("Usage: remove <playlist> <song_idx>");
                }
            },

            _ => {
                println!("Unknown command: '{}', type 'help' for commands", cmd);
            },
        }
    }

    // Auto-save on exit
    if let Err(e) = library.save() {
        println!("Failed to save library: {:?}", e);
    }
}
