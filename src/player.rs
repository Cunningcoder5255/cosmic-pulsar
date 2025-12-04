// use crate::HEIGHT;
use crate::app::Message;
use crate::page::albums_page::Album;
use crate::song::Song;
use cosmic::Element;
use cosmic::iced::Length;
use cosmic::iced_core::Alignment;
use cosmic::theme;
use cosmic::widget::*;
use rodio::Decoder;
use rodio::stream::OutputStream;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    PlaySong(Song),           // Plays a specific song, clearing the playlist
    PlayAlbum((Album, Song)), // Plays an album, clearing the playlist
    Play,                     // Start playback
    Pause,                    // Stop playback, keeping playlist
    Update,                   // Updates the playing song and the progress
    ProgressSlider(f32),      // Updates the sink to play the current song at the appropriate time
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
            PlayerMessage::Play => {
                self.play();
            }
            PlayerMessage::Pause => {
                self.pause();
            }
            PlayerMessage::Update => {
                self.sync();
            }
            PlayerMessage::ProgressSlider(progress_input) => {
                eprintln!("Going to {:#?} seconds in source.", progress_input);
                self.sink
                    .try_seek(Duration::from_secs_f32(progress_input))
                    .expect("Could not seek through given source.");
                self.sync();
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
    /// Updates the internal player state to sync with the sink
    pub fn sync(&mut self) {
        // if self.song_index == 0 {
        //     return;
        // }
        // eprintln!("index before: {:#?}", self.song_index);
        let mut prior_duration = Duration::from_secs(0);
        let pos = self.sink.get_pos();

        // Loop over song durations until we reach the song before the current one, updating the internal song_index as we go
        for (i, song) in self.playlist.iter().enumerate() {
            let temp_duration = prior_duration + song.duration;
            // If we get to the song we are currently playing, break from the loop
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
    /// Clears the queue and plays the given song
    pub fn play_song(&mut self, song: Song) {
        self.song_index = 0;
        self.sink.stop();
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
        // eprintln!("{:#?}", index - self.song_index);
        if index > 0 {
            for _ in 0..(index - self.song_index) {
                // eprintln!("Skipping one song.");
                self.sink.skip_one();
            }
            // self.sink.skip_one();
        }
        // eprintln!("Playlist: {:#?}", self.playlist);
        // eprintln!("Sink empty: {:#?}", self.sink.empty());
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
            .into();
        let song_title = container(text(song.title.clone())).center_x(Length::Fill);
        let song_progress: Element<Message> = slider(
            0.0..=song.duration.as_secs_f32(),
            self.progress.as_secs_f32(),
            |v| Message::Player(PlayerMessage::ProgressSlider(v)),
        )
        .step(0.01)
        // .height(10)
        .into();
        let song_progress_widget: Element<Message> = row::with_capacity(3)
            .push(text(self.progress.string_mins_secs()))
            .push(song_progress)
            .push(text(song.duration.string_mins_secs()))
            .align_y(Alignment::Center)
            .height(20)
            .spacing(spacing)
            .into();
        // TODO: Add fallback icons
        let play_pause_button: Element<Message> = if self.playing {
            // let icon = icon::from_name("media-playback-stop"); Doesn't work ig
            let icon: Element<Message> = svg(svg::Handle::from_memory(
                include_bytes!("../resources/svg/pause.svg").as_slice(),
            ))
            .into();
            button::custom(icon)
                .height(50)
                .width(50)
                .on_press(Message::Player(PlayerMessage::Pause))
                .class(theme::Button::Suggested)
                .into()
        } else {
            // let icon = icon::from_name("media-playback-start")
            let icon: Element<Message> = svg(svg::Handle::from_memory(
                include_bytes!("../resources/svg/play.svg").as_slice(),
            ))
            .into();
            button::custom(icon)
                .height(50)
                .width(50)
                .on_press(Message::Player(PlayerMessage::Play))
                .class(theme::Button::Suggested)
                .into()
        };
        let play_pause = container(play_pause_button)
            // .center_y(60)
            .center_x(Length::Fill);
        let playing_song = container(
            column::with_capacity(4)
                .push(song_image)
                .push(song_title)
                .push(song_progress_widget)
                .push(play_pause)
                .spacing(spacing),
        );

        let mut playlist_songs: Vec<Element<Message>> = vec![];
        for song in self.playlist.iter() {
            let button = button::custom(song.display())
                .on_press(Message::Player(PlayerMessage::PlaySong(song.clone())));
            playlist_songs.push(button.into());
        }

        let playlist_container = scrollable(
            column::with_children(playlist_songs).spacing(spacing), // .padding(spacing),
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
        let file_unbuf = File::open(song.path.clone()).unwrap();
        let song_length = file_unbuf.metadata().unwrap().len();
        let song_file = BufReader::new(file_unbuf);
        // Decode that sound file into a rodio source
        let source = Decoder::builder()
            .with_data(song_file)
            .with_byte_len(song_length)
            .with_seekable(true)
            .with_coarse_seek(true)
            .with_gapless(true)
            .build()
            .unwrap();
        self.append(source);

        eprintln!("Adding {:#?} to playlist.", song.title);
    }
}

trait DurationDisplayExt {
    /// Returns a string as minutes:seconds
    fn string_mins_secs(&self) -> String;
}
impl DurationDisplayExt for Duration {
    fn string_mins_secs(&self) -> String {
        let duration_int = self.as_secs();
        let min = duration_int / 60;
        let sec = duration_int % 60;
        let sec_text: String;
        // Ensure 1 is displayed as 01
        if sec <= 9 {
            sec_text = "0".to_string() + &sec.to_string();
        } else {
            sec_text = sec.to_string();
        }

        min.to_string() + ":" + &sec_text
    }
}
