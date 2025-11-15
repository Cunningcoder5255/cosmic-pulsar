use crate::app::Message;
use crate::page::Page;

#[derive(Debug, Clone)]
pub enum FilesPageMessage {
    TODO(),
}

pub struct FilesPage {}

impl Page for FilesPage {
    fn view(&self) -> cosmic::Element<'_, Message> {
        todo!();
    }
    fn update(&mut self, message: Message) -> Option<Box<dyn Page>> {
        todo!();
    }
}
