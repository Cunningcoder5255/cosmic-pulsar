use crate::page::*;
use crate::player;
use crate::player::Player;
use cosmic::iced::futures::SinkExt;
use cosmic::iced::time::Duration;
use cosmic::iced_futures;
use player::PlayerMessage;
use slotmap::Key;
use std::error::Error;
use std::path::PathBuf;
extern crate tokio;
use albums_page::AlbumsPage;
use artists_page::ArtistsPage;
use cosmic::iced::Subscription;
use cosmic::widget::nav_bar;
use cosmic::widget::pane_grid;
use cosmic::widget::pane_grid::Axis;
use cosmic::{self, Task};
use std::env;
use std::path;
extern crate rodio;

#[derive(Debug, Clone)]
pub enum Message {
    FilesPage(files_page::FilesPageMessage),
    AlbumsPage(albums_page::AlbumsPageMessage),
    ArtistsPage(artists_page::ArtistsPageMessage),
    Player(player::PlayerMessage),
}

enum Pane {
    Content,
    Player,
}

enum IdentifyPage {
    AlbumsPage,
    ArtistsPage,
}

pub struct App {
    page: Box<dyn Page>,
    core: cosmic::Core,
    nav_bar: nav_bar::Model,
    pane_state: pane_grid::State<Pane>,
    pub player: Player,
    music_dir: PathBuf,
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
            //             // idk what im doing this prob needs to be refactored when i do ^
            // .icon(icon::from_name("applications-science-symbolic"))
            .data::<u16>(0)
            .activate();
        nav_bar.insert().text("Artists").data::<u16>(0);

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
            page: albums_page,
            nav_bar,
            core,
            pane_state,
            player: Player::default(),
            music_dir,
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
            Message::Player(player_message) => {
                self.player.update(player_message);
                return cosmic::Task::none();
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
    /// Subscription, primarily for updating the song progress bar as time passes
    fn subscription(&self) -> Subscription<Message> {
        return Subscription::run(|| {
            iced_futures::stream::channel(1, |mut emitter| async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));

                loop {
                    interval.tick().await;
                    _ = emitter.send(Message::Player(PlayerMessage::Update)).await;
                }
            })
        });
    }
    /// Enable the nav bar to appear in your application when `Some`.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_bar)
    }

    /// Activate the nav item when selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> cosmic::Task<cosmic::Action<Message>> {
        // Activate the page in the model.
        self.nav_bar.activate(id);
        let task;
        (self.page, task) = self
            .page_from_id(id.data().as_ffi())
            .expect("Could not create page.");
        // let task = cosmic::Task::none();
        // let page;
        // match id {
        //     id if id == "Albums".into() => {
        //         let (page, task) =
        //             AlbumsPage::new(&self.music_dir).expect("Could not find albums: ");
        //     }
        //     "Artists" => {
        //         page = Box::new(ArtistsPage::default());
        //     }
        // }
        task
    }
}

impl App {
    fn page_from_id(
        &self,
        page_id: u64,
    ) -> Result<(Box<dyn Page>, Task<cosmic::Action<Message>>), Box<dyn Error>> {
        let page = match page_id {
            0 => AlbumsPage::new(&self.music_dir),
            1 => ArtistsPage::new(&self.music_dir),
            _ => panic!("Value not in range"),
        };
        page
    }
}
