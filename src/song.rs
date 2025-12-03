// pub use crate::page::albums_page::Album;
// use cosmic::iced;
use cosmic::iced::{Alignment, Length};
extern crate cosmic;
use crate::HEIGHT;
use crate::app::Message;
use crate::page::card_style;
use cosmic::widget::*;
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use std::cmp::Ordering;
use std::hash::Hash;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Song {
    pub title: String,
    pub artist: Option<String>,
    pub album_title: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub picture: image::Handle,
    pub path: PathBuf,
    pub index: Option<u32>,
    pub duration: Duration,
}
impl Ord for Song {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.index > other.index {
            return Ordering::Greater;
        } else if self.index == other.index {
            return Ordering::Equal;
        } else {
            return Ordering::Less;
        }
    }
}
impl PartialOrd for Song {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.index > other.index {
            return Some(Ordering::Greater);
        } else if self.index == other.index {
            return Some(Ordering::Equal);
        } else {
            return Some(Ordering::Less);
        }
    }
}
impl Song {
    pub fn new(
        title: String,
        artist: Option<String>,
        album_title: Option<String>,
        genre: Option<String>,
        year: Option<u32>,
        picture: image::Handle,
        path: &Path,
        index: Option<u32>,
        duration: Duration,
    ) -> Self {
        Self {
            title,
            artist,
            genre,
            album_title,
            year,
            picture,
            path: path.to_path_buf(),
            index,
            duration,
        }
    }
    pub fn display(&self) -> cosmic::Element<'_, Message> {
        let space = cosmic::theme::spacing().space_s;
        let picture = image(self.picture.clone());
        let name = text(self.title.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Center);
        let index = text(
            self.index
                .map(|i| i.to_string())
                .unwrap_or_else(|| "".to_string()),
        )
        .align_y(Alignment::Center)
        .height(Length::Fill);
        let container = container(
            row::with_capacity::<Message>(3)
                .push(picture)
                .push(name)
                .push(index)
                .spacing(space),
        )
        .style(card_style)
        .height(HEIGHT);

        container.into()
    }
    pub async fn from_path(path: PathBuf) -> Result<Self, lofty::error::LoftyError> {
        let mut stderr_lock = std::io::stderr().lock();
        let _ = writeln!(stderr_lock, "Creating song from path: {:#?}", path);
        let lofty_file = lofty::read_from_path(&path)?;
        let duration = lofty_file.properties().duration();
        let file_tag = lofty_file
            .primary_tag()
            .ok_or_else(|| lofty::error::LoftyError::new(lofty::error::ErrorKind::FakeTag))?;
        // Most unreadable line of code I've ever written
        // Attempts to parse file name, if it can't returns lofty error TextDecode
        let file_name = path
            .file_name()
            .ok_or_else(|| {
                lofty::error::LoftyError::new(lofty::error::ErrorKind::TextDecode(
                    "Could not decode file name.",
                ))
            })?
            .to_str()
            .ok_or_else(|| {
                lofty::error::LoftyError::new(lofty::error::ErrorKind::TextDecode(
                    "Could not convert file name to rust string.",
                ))
            })?;
        // Set title either to the title tag or the file name
        let title = file_tag.title().unwrap_or_else(|| file_name.into());
        let album_title = file_tag.album().map(|title| title.to_string());
        let picture = file_tag.pictures().to_vec().pop();
        let picture_handle: image::Handle;
        if let Some(picture) = picture {
            picture_handle = image::Handle::from_bytes(picture.into_data());
        } else {
            picture_handle = image::Handle::from_bytes(
                include_bytes!("../resources/images/albumplaceholder.png").as_slice(),
            );
        }
        let index = file_tag.track();
        let artist = file_tag.artist().map(|artist| artist.to_string());
        let genre = file_tag.genre().map(|genre| genre.to_string());
        let year = file_tag.year();
        let _ = writeln!(stderr_lock, "Done creating song: {:#?}", path);

        Ok(Self::new(
            title.to_string(),
            artist,
            album_title,
            genre,
            year,
            picture_handle,
            &path,
            index,
            duration,
        ))
    }
}
