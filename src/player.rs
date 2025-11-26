use crate::app::Message;
use crate::song::Song;
use cosmic::Element;
use cosmic::widget::*;
use std::time::Duration;

pub struct Player {
    song_index: usize,
    playlist: Vec<Song>,
    playing: bool,
    shuffle: bool,
    progress: Duration,
}

impl Player {
    pub fn new(
        song_index: impl Into<usize>,
        playlist: Vec<Song>,
        playing: bool,
        shuffle: bool,
        progress: Duration,
    ) -> Player {
        Self {
            song_index: song_index.into(),
            playlist,
            playing,
            shuffle,
            progress,
        }
    }
    pub fn default() -> Player {
        Self {
            song_index: 0,
            playlist: vec![],
            playing: false,
            shuffle: false,
            progress: Duration::from_secs(0),
        }
    }
    /// Begin playing the next song in the playlist
    pub fn play(&mut self) {
        todo!()
    }
    /// Draws the content for the music player
    /// Split into two sections, the top section which shows the current song, and the bottom section which shows the playlist
    pub fn view(&self) -> Element<'_, Message> {
        let Some(song) = self.playlist.get(self.song_index) else {
            return text("Could not view song").into();
        };
        let song_image: Element<Message> = image(song.picture.clone()).into();
        let song_progress: Element<Message> = progress_bar(
            0.0..=song.duration.as_secs_f32(),
            self.progress.as_secs_f32(),
        )
        .into();

        container(
            column::with_capacity(2)
                .push(song_image)
                .push(song_progress),
        )
        .into()
    }
}
