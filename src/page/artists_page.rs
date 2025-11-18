use crate::app::Message;
use crate::page::Page;

#[derive(Debug, Clone)]
pub enum ArtistsPageMessage {
    TODO(),
}

pub struct ArtistsPage {}

impl Page for ArtistsPage {
    fn view(&self) -> cosmic::Element<'_, Message> {
        todo!();
    }
    fn update(
        &mut self,
        message: Message,
    ) -> (cosmic::Task<cosmic::Action<Message>>, Option<Box<dyn Page>>) {
        todo!();
    }
}
