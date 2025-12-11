use crate::app::Message;
use crate::song::Song;
extern crate walkdir;
use cosmic::{Action, Task};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SongLibrary {
    songs: HashSet<Song>,
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

        let mut tasks: Vec<cosmic::Task<Option<Song>>> = vec![];

        for path in paths {
            let _ = writeln!(lock, "Pushing task for: {:#?}", path);
            tasks.push(cosmic::Task::perform(Song::from_path(path), |song| {
                song.ok()
            }));
        }

        tasks
    }
    pub async fn add_song(&mut self, song: Song) {
        self.songs.insert(song);
    }
    pub fn default() -> Self {
        Self {
            songs: vec![].into_iter().collect(),
            show_album: None,
        }
    }
}
