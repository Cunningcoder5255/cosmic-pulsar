use crate::app::Message;
use cosmic;
pub mod albums_page;
pub mod artists_page;
pub mod files_page;

// pub enum Pages {
//     AlbumsPage(albums_page::AlbumsPage),
//     ArtistsPage(artists_page::ArtistsPage),
//     FilesPage(files_page::FilesPage),
// }

pub trait Page {
    fn update(
        &mut self,
        message: Message,
    ) -> (cosmic::Task<cosmic::Action<Message>>, Option<Box<dyn Page>>);
    fn view(&self) -> cosmic::Element<'_, Message>;
}

/// The style of containers for things like the album and song cards
/// Used to use smaller borders that aren't going to cut off content
pub fn card_style(_theme: &cosmic::Theme) -> cosmic::widget::container::Style {
    // let radius = theme.cosmic().corner_radii.radius_m;
    let radius: [u16; 4] = [2; 4];
    let container_style = cosmic::widget::container::Style::default()
        .border(cosmic::iced::Border::default().rounded(radius));
    container_style
}
