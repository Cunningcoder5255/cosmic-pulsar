use crate::page::*;
use albums_page::AlbumsPage;
use cosmic;
use cosmic::widget::nav_bar;
use std::env;
use std::path;

#[derive(Debug, Clone)]
pub enum Message {
    FilesPage(files_page::FilesPageMessage),
    AlbumsPage(albums_page::AlbumsPageMessage),
    ArtistsPage(artists_page::ArtistsPageMessage),
}

pub struct App {
    page: Box<dyn Page>,
    core: cosmic::Core,
    nav_bar: nav_bar::Model,
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
            music_dir = path::PathBuf::from("/");
        }
        let app = Self {
            page: Box::new(AlbumsPage::new(&music_dir).expect("Could not find albums: ")),
            nav_bar,
            core,
        };
        (app, cosmic::Task::none())
    }
    fn core(&self) -> &cosmic::Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }
    // View the state of the application
    fn view(&self) -> cosmic::Element<'_, Message> {
        self.page.view()
    }
    // Update the state of the application with messages from view
    fn update(&mut self, message: Message) -> cosmic::Task<cosmic::Action<Message>> {
        let page = self.page.update(message);
        if let Some(p) = page {
            self.page = p;
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
