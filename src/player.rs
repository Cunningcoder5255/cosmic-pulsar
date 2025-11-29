use crate::app::Message;
use crate::song::Song;
use cosmic::Element;
use cosmic::widget::*;
use rodio::Decoder;
use rodio::stream::OutputStream;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub struct Player {
    song_index: usize,
    playlist: Vec<Song>,
    playing: bool,
    shuffle: bool,
    progress: Duration,
    stream_handle: OutputStream, // Keep stream handle alive to continue playback
    sink: rodio::Sink,           // Keep audio sink alive to continue playback
}

impl Player {
    pub fn new(
        song_index: impl Into<usize>,
        playlist: Vec<Song>,
        playing: bool,
        shuffle: bool,
        progress: Duration,
        stream_handle: OutputStream,
        sink: rodio::Sink,
    ) -> Player {
        Self {
            song_index: song_index.into(),
            playlist,
            playing,
            shuffle,
            progress,
            stream_handle,
            sink,
        }
    }
    pub fn default() -> Player {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .expect("Could not create audio sink: ");
        Self {
            song_index: 0,
            playlist: vec![],
            playing: false,
            shuffle: false,
            progress: Duration::from_secs(0),
            sink: rodio::Sink::connect_new(&stream_handle.mixer()),
            stream_handle,
        }
    }
    /// Clears the queue and plays the given song
    pub fn play_song(&mut self, song: Song) {
        self.song_index = 0;
        self.playlist = vec![song];
        self.play(0);
    }
    /// Begin playing the song at the given index in the playlist
    pub fn play(&mut self, index: usize) {
        // Get the song at the given index
        let Some(song) = self.playlist.get(index) else {
            return;
        };

        // Load the song file into memory
        let song_file = BufReader::new(File::open(song.path.clone()).unwrap());
        // Decode that sound file into a rodio source
        let source = Decoder::try_from(song_file).unwrap();
        // Clear the sink instead of adding song to the end of the list
        self.sink.clear();
        // Add that source to the audio sink
        self.sink.append(source);
    }
    /// Begin playing the next song in the playlist
    pub fn play_next(&mut self) {
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
