use crate::page::*;
use crate::player::Player;
use crate::song::Album;
use crate::song::Song;
use albums_page::AlbumsPage;
use cosmic;
use cosmic::widget::nav_bar;
use cosmic::widget::pane_grid;
use cosmic::widget::pane_grid::Axis;
use std::env;
use std::path;
extern crate rodio;

#[derive(Debug, Clone)]
pub enum Message {
    FilesPage(files_page::FilesPageMessage),
    AlbumsPage(albums_page::AlbumsPageMessage),
    ArtistsPage(artists_page::ArtistsPageMessage),
    PlaySong(Song),
    PlayAlbum((Album, Song)),
}

enum Pane {
    Content,
    Player,
}

pub struct App {
    page: Box<dyn Page>,
    core: cosmic::Core,
    nav_bar: nav_bar::Model,
    pane_state: pane_grid::State<Pane>,
    pub player: Player,
}

impl cosmic::Application for App {
    const APP_ID: &str = "com.github.cunningcoder5255.cosmic-pulsar";
    // Async executor - single or multi
    type Executor = cosmic::executor::Default;
    // App flags - config options
    type Flags = ();
    type Message = Message;

    fn init(core: cosmic::Core, _flags: ()) -> (Self, cosmic::Task<cosmic::Action<Message>>) {
        let mut nav_bar = nav_bar::Model::default();

        nav_bar
            .insert()
            .text("Albums")
            // .data::<Box<dyn Page>>(Box::new(AlbumsPage::default()))
            // idk what im doing this prob needs to be refactored when i do ^
            // .icon(icon::from_name("applications-science-symbolic"))
            .activate();
        nav_bar.insert().text("Artists");

        let music_dir: path::PathBuf;
        if let Some(mut home_dir) = env::home_dir() {
            home_dir.push("Music");
            music_dir = home_dir;
        } else {
            // Slow and kind of stupid
            music_dir = path::PathBuf::from("/");
        }
        let (albums_page, task) = AlbumsPage::new(&music_dir).expect("Could not find albums: ");
        // Initialize pane state
        let (mut pane_state, pane) = pane_grid::State::new(Pane::Content);
        pane_state.split(Axis::Vertical, pane, Pane::Player);

        // let player = Player::default();
        let app = Self {
            page: Box::new(albums_page),
            nav_bar,
            core,
            pane_state,
            player: Player::default(),
        };
        (app, task)
    }
    fn core(&self) -> &cosmic::Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }
    // View the state of the application
    fn view(&self) -> cosmic::Element<'_, Message> {
        pane_grid(&self.pane_state, |_pane, state, _is_maximized| {
            pane_grid::Content::new(match state {
                Pane::Player => self.player.view(),
                Pane::Content => self.page.view(),
            })
        })
        .into()
    }
    // Update the state of the application with messages from view
    fn update(&mut self, message: Message) -> cosmic::Task<cosmic::Action<Message>> {
        match message {
            Message::PlaySong(song) => {
                self.player.play_song(song);
                return cosmic::Task::none();
            }
            Message::PlayAlbum((album, song)) => {
                self.player.clear_playlist();
                self.player
                    .add_to_playlist(album.get_songs().clone().into_iter().collect());
                self.player.play_index(album.get_song_index(&song).unwrap());
            }
            _ => {
                let page = self.page.update(message);
                if let (task, Some(p)) = page {
                    self.page = p;
                    return task;
                }
            }
        }
        cosmic::Task::none()
    }
    /// Enable the nav bar to appear in your application when `Some`.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_bar)
    }

    /// Activate the nav item when selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> cosmic::Task<cosmic::Action<Message>> {
        // Activate the page in the model.
        self.nav_bar.activate(id);
        cosmic::Task::none()
    }
}
