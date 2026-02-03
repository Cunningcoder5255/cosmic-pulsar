use crate::app::Message;
use crate::song::Song;
use std::borrow::Cow;
use std::collections::BTreeMap;
extern crate walkdir;
use cosmic::{Action, Task};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct SongLibrary {
    songs: Vec<Song>,
    pub show_album: Option<String>,
}
impl SongLibrary {
    pub fn populate(path: PathBuf) -> Vec<cosmic::Task<Option<Song>>> {
        let mut lock = std::io::stderr().lock();
        let _ = writeln!(lock, "{:#?}", path);
        let mut paths: Vec<PathBuf> = vec![];

        for entry in WalkDir::new(path)
            .contents_first(true)
            .follow_links(true)
            .into_iter()
            .filter(|e| {
                if let Ok(e) = e {
                    let Ok(metadata) = Path::metadata(e.path()) else {
                        return false;
                    };
                    return metadata.is_file();
                } else {
                    return false;
                };
            })
        {
            let _ = writeln!(lock, "Inspecting directory/file: {:#?}", entry);
            let Ok(entry) = entry else { continue }; // Skip bad paths

            paths.push(entry.into_path());
        }
        eprintln!("paths are: {:#?}", paths);

        let mut tasks: Vec<cosmic::Task<Option<Song>>> = vec![];

        for path in paths {
            let _ = writeln!(lock, "Pushing task for: {:#?}", path);
            tasks.push(cosmic::Task::perform(Song::from_path(path), |song| {
                // eprintln!("{:#?}", song);
                song.ok()
            }));
        }

        tasks
    }
    pub fn add_song(&mut self, song: Song) {
        // eprintln!("Adding song {:#?}", song);
        self.songs.push(song);
    }
    pub fn default() -> Self {
        Self {
            songs: vec![].into_iter().collect(),
            show_album: None,
        }
    }
    pub fn get_album(&self, album: &str) -> Vec<Song> {
        // eprintln!("getting albums");
        let mut songs: Vec<Song> = vec![];
        for song in self.songs.iter() {
            // eprintln!("{:#?}, {:#?}", song.album_title, album.to_string());
            if song.album_title == Some(album.to_string()) {
                songs.push(song.clone());
            }
        }

        songs
    }
    pub fn get_albums(&self) -> BTreeMap<String, Vec<Song>> {
        let mut songs: BTreeMap<String, Vec<Song>> = BTreeMap::new();
        // Loop over library songs
        for song in self.songs.iter() {
            // Get title, skip if none
            let Some(album_title) = song.album_title.clone() else {
                // eprintln!("No album title, not categorizing.");
                continue;
            };
            match songs.get_mut(&album_title) {
                // If album exists, add current song to it
                Some(entry) => {
                    entry.push(song.clone());
                    // eprintln!("Adding song to album");
                }
                // If entry does not exist, add song to new album
                None => {
                    songs.insert(album_title, vec![song.clone()]);
                    // eprintln!("Adding song to library");
                }
            }
        }
        songs
    }
}
