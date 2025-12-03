use crate::HEIGHT;
use crate::app::Message;
use crate::page::albums_page::Album;
use crate::song::Song;
use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced_core::Alignment;
use cosmic::widget::*;
use rodio::Decoder;
use rodio::stream::OutputStream;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    PlaySong(Song),
    PlayAlbum((Album, Song)),
    Update,
}

pub struct Player {
    song_index: usize,
    playlist: Vec<Song>,
    playing: bool,
    shuffle: bool,
    progress: Duration,
    _stream_handle: OutputStream, // Keep stream handle alive to continue playback
    sink: rodio::Sink,            // Keep audio sink alive to continue playback
}

impl Player {
    pub fn new(
        song_index: impl Into<usize>,
        playlist: Vec<Song>,
        playing: bool,
        shuffle: bool,
        progress: Duration,
        _stream_handle: OutputStream,
        sink: rodio::Sink,
    ) -> Player {
        Self {
            song_index: song_index.into(),
            playlist,
            playing,
            shuffle,
            progress,
            _stream_handle,
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
            _stream_handle: stream_handle,
        }
    }
    /// Handles cosmic messages
    pub fn update(&mut self, message: PlayerMessage) {
        match message {
            PlayerMessage::PlaySong(song) => {
                self.play_song(song);
            }
            PlayerMessage::PlayAlbum((album, song)) => {
                self.clear_playlist();
                self.add_to_playlist(album.get_songs().clone().into_iter().collect());
                self.play_index(album.get_song_index(&song).unwrap());
            }
            PlayerMessage::Update => {
                if self.song_index == 0 {
                    return;
                }
                // eprintln!("index before: {:#?}", self.song_index);
                let mut prior_duration = Duration::from_secs(0);
                let pos = self.sink.get_pos();

                // Loop over song durations until we reach the song before the current one, updating the internal song_index as we go
                // while prior_duration < pos {
                //     self.playlist.iter().enumerate().for_each(|(i, song)| {
                //         prior_duration += song.duration;
                //         self.song_index = i;
                //     })
                // }
                for (i, song) in self.playlist.iter().enumerate() {
                    let temp_duration = prior_duration + song.duration;
                    if temp_duration > pos {
                        break;
                    }
                    prior_duration = temp_duration;
                    self.song_index = i;
                }

                // eprintln!("index: {:#?}", self.song_index);
                // eprintln!("prior duration: {:#?}", prior_duration);
                // eprintln!("pos: {:#?}", pos);
                self.progress = pos - prior_duration;
            }
        }
    }
    /// Returns whether or not the Player is playing a song
    pub fn is_playing(&self) -> bool {
        self.playing
    }
    /// Adds the given songs to the queue
    pub fn add_to_playlist(&mut self, mut songs: Vec<Song>) {
        // Add songs to sink playlist
        for song in songs.iter() {
            self.sink.add_song(&song);
            // eprintln!("Adding {:#?} to playlist.", song);
        }

        self.playlist.append(&mut songs);
    }
    /// Clears the playlist
    pub fn clear_playlist(&mut self) {
        self.song_index = 0;
        self.playlist = vec![];
        self.sink.stop();
    }
    /// Clears the queue and plays the given song
    pub fn play_song(&mut self, song: Song) {
        self.song_index = 0;
        self.sink.add_song(&song);
        self.playlist = vec![song];
        self.play();
    }
    pub fn play(&mut self) {
        self.sink.play();
        self.playing = true;
    }
    pub fn pause(&mut self) {
        self.sink.pause();
        self.playing = false;
    }
    /// Begin playing the song at the given index in the playlist
    /// If the sink does not start from playlist index 0, this produce unexpected results
    pub fn play_index(&mut self, index: usize) {
        // Skip all songs up to index
        eprintln!("{:#?}", index - self.song_index);
        if index > 0 {
            for _ in 0..(index - self.song_index) {
                eprintln!("Skipping one song.");
                self.sink.skip_one();
            }
            // self.sink.skip_one();
        }
        // eprintln!("Playlist: {:#?}", self.playlist);
        eprintln!("Sink empty: {:#?}", self.sink.empty());
        // Play
        self.play();
        // Set the index
        self.song_index = index;
    }
    /// Begin playing the next song in the playlist
    pub fn play_next(&mut self) {
        self.sink.skip_one();
    }
    /// Draws the content for the music player
    /// Split into two sections, the top section which shows the current song, and the bottom section which shows the playlist
    pub fn view(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing().space_s;
        // let spacing_l = cosmic::theme::spacing().space_l;
        // let spacing_s = cosmic::theme::spacing().space_xxs;

        // Get the playing song or return default
        let Some(song) = self.playlist.get(self.song_index) else {
            return text("No song playing.").into();
        };
        let song_image: Element<Message> = container(image(song.picture.clone()))
            .center_x(Length::Fill)
            .max_height(400)
            .padding(spacing)
            .into();
        let song_title = text(song.title.clone());
        let song_progress: Element<Message> = progress_bar(
            0.0..=song.duration.as_secs_f32(),
            self.progress.as_secs_f32(),
        )
        .height(10)
        .into();
        let playing_song = container(
            column::with_capacity(3)
                .push(song_image)
                .push(song_title)
                .push(song_progress),
        );

        let mut playlist_songs: Vec<Element<Message>> = vec![];
        for song in self.playlist.iter() {
            let button = button::custom(song.display())
                .on_press(Message::Player(PlayerMessage::PlaySong(song.clone())));
            playlist_songs.push(button.into());
        }

        let playlist_container = scrollable(
            column::with_children(playlist_songs)
                .spacing(spacing)
                .padding(spacing),
        );

        container(
            column::with_capacity(2)
                .push(playing_song)
                .push(playlist_container)
                .padding(spacing)
                .spacing(spacing),
        )
        .into()
    }
}

trait SinkSongExt {
    fn add_song(&mut self, song: &Song);
}

impl SinkSongExt for rodio::Sink {
    fn add_song(&mut self, song: &Song) {
        // Load the song file into memory
        let song_file = BufReader::new(File::open(song.path.clone()).unwrap());
        // Decode that sound file into a rodio source
        let source = Decoder::try_from(song_file).unwrap();
        self.append(source);

        eprintln!("Adding {:#?} to playlist.", song);
    }
}
