use crate::app::{self, Message};
use crate::page::Page;
use crate::song_library::SongLibrary;
use cosmic::widget::*;
use cosmic::{Action, Task};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum ArtistsPageMessage {
    TODO(),
}

pub struct ArtistsPage {
    song_library: SongLibrary,
}

impl Page for ArtistsPage {
    fn view(&self) -> cosmic::Element<'_, Message> {
        text("Artists page.").into()
    }
    fn update(
        &mut self,
        message: Message,
    ) -> (cosmic::Task<cosmic::Action<Message>>, Option<Box<dyn Page>>) {
        return (cosmic::Task::none(), None);
    }
}

impl ArtistsPage {
    pub fn new(
        music_dir: &Path,
    ) -> Result<(Box<dyn Page>, cosmic::Task<cosmic::Action<Message>>), Box<dyn Error>> {
        let song_library = SongLibrary::default();
        // let populate_task = cosmic::Task::batch(SongLibrary::populate(music_dir.into()))
        // .map(|o| cosmic::Action::App(Message::AlbumsPage(AlbumsPageMessage::Populate(o))));
        let populate_task = cosmic::Task::none();

        Ok((Box::new(ArtistsPage { song_library }), populate_task))
    }
}
