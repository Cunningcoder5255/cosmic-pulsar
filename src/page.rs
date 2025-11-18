use crate::app::Message;
use cosmic;
pub mod albums_page;
pub mod artists_page;
pub mod files_page;

pub enum Pages {
    AlbumsPage(albums_page::AlbumsPage),
    ArtistsPage(artists_page::ArtistsPage),
    FilesPage(files_page::FilesPage),
}
pub trait Page {
    fn update(
        &mut self,
        message: Message,
    ) -> (cosmic::Task<cosmic::Action<Message>>, Option<Box<dyn Page>>);
    fn view(&self) -> cosmic::Element<'_, Message>;
}
