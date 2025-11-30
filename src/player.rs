use crate::HEIGHT;
use crate::app::Message;
use crate::song::Song;
use cosmic::Element;
use cosmic::iced::Length;
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
    /// Adds the given songs to the queue
    pub fn add_to_playlist(&mut self, mut songs: Vec<Song>) {
        self.playlist.append(&mut songs);
        // Add songs to sink playlist
        for song in songs {
            // Load the song file into memory
            let song_file = BufReader::new(File::open(song.path.clone()).unwrap());
            // Decode that sound file into a rodio source
            let source = Decoder::try_from(song_file).unwrap();
            self.sink.append(source);
            eprintln!("Adding {:#?} to playlist.", song);
        }
    }
    /// Clears the playlist
    pub fn clear_playlist(&mut self) {
        self.playlist = vec![];
        self.sink.stop();
    }
    /// Clears the queue and plays the given song
    pub fn play_song(&mut self, song: Song) {
        self.song_index = 0;
        self.playlist = vec![song];
        self.play_index(0);
    }
    pub fn play(&mut self) {
        self.sink.play();
        self.playing = true;
    }
    pub fn pause(&mut self) {
        self.sink.pause();
        self.playing = false;
    }
    /// Updates the sink to queue the next songs in the playlist
    fn update_sink(&mut self) {
        self.sink.clear();
        for song in self.playlist[self.song_index..].iter() {
            self.sink.add_song(song);
        }
    }
    /// Begin playing the song at the given index in the playlist
    pub fn play_index(&mut self, index: usize) {
        // Set the index
        self.song_index = index;
        // Update the sink to play from the index
        self.update_sink();
        // Play
        self.sink.play();
    }
    /// Begin playing the next song in the playlist
    pub fn play_next(&mut self) {
        todo!()
    }
    /// Draws the content for the music player
    /// Split into two sections, the top section which shows the current song, and the bottom section which shows the playlist
    pub fn view(&self) -> Element<'_, Message> {
        let Some(song) = self.playlist.get(self.song_index) else {
            return text("No song playing.").into();
        };
        let song_image: Element<Message> = image(song.picture.clone()).into();
        let song_progress: Element<Message> = progress_bar(
            0.0..=song.duration.as_secs_f32(),
            self.progress.as_secs_f32(),
        )
        .into();
        let mut playlist_songs: Vec<Element<Message>> = vec![];
        for song in self.playlist.iter() {
            let picture = image(&song.picture);
            let title = text(&song.title).width(Length::Fill);
            let index = text(
                song.index
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "".to_string()),
            );
            let song_container = row::with_capacity::<Message>(3)
                .push(picture)
                .push(title)
                .push(index)
                .height(HEIGHT);
            playlist_songs.push(song_container.into());
        }

        let playlist_container = scrollable(column::with_children(playlist_songs));

        container(
            column::with_capacity(3)
                .push(song_image)
                .push(song_progress)
                .push(playlist_container),
        )
        .into()
    }
}

trait SinkSong {
    fn add_song(&mut self, song: &Song);
}

impl SinkSong for rodio::Sink {
    fn add_song(&mut self, song: &Song) {
        // Load the song file into memory
        let song_file = BufReader::new(File::open(song.path.clone()).unwrap());
        // Decode that sound file into a rodio source
        let source = Decoder::try_from(song_file).unwrap();
        self.append(source);
    }
}
