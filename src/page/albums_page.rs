// use crate::app::App;
extern crate rayon;
extern crate walkdir;
use crate::app::Message;
use crate::page::Page;
use cosmic;
use cosmic::Element;
use rayon::prelude::*;
use walkdir::WalkDir;
// use cosmic::iced::Alignment;
use cosmic::iced::Length;
use cosmic::widget::*;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use std::error;
use std::path::{self, PathBuf};

#[derive(Debug, Clone)]
pub enum AlbumsPageMessage {
    ShowAlbum(Album),
}

struct AlbumsLibrary {
    albums: Vec<Album>,
}
impl AlbumsLibrary {
    // pub fn new(albums: Vec<Album>) -> Self {
    //     AlbumsLibrary { albums }
    // }
    pub fn default() -> Self {
        AlbumsLibrary { albums: vec![] }
    }
    pub fn get_albums(&self) -> &Vec<Album> {
        &self.albums
    }
    pub fn populate(&mut self, path: impl AsRef<path::Path>) -> Result<(), Box<dyn error::Error>> {
        dhat::ad_hoc_event(1);
        // Go over the entries in the directory
        for entry in WalkDir::new(path).contents_first(true).follow_links(true) {
            let entry = entry?; // Return errors with entry
            // Push files to the dir
            if entry.path().is_dir() {
                continue;
            }
            // If lofty can't read the file, continue
            let lofty_path: lofty::file::TaggedFile;
            if let Ok(ok_lofty_path) = lofty::read_from_path(entry.path()) {
                lofty_path = ok_lofty_path;
            } else {
                continue;
            };
            // If file has no tags, skip it
            let tag = lofty_path.primary_tag();
            if tag.is_none() {
                continue;
            }

            let album_title = tag.unwrap().album().unwrap_or_default().to_string();
            let picture = tag.unwrap().pictures().to_vec().pop();
            let picture_handle: image::Handle;
            if let Some(picture) = picture {
                picture_handle = image::Handle::from_bytes(picture.into_data());
            } else {
                picture_handle = image::Handle::from_bytes(
                    include_bytes!("../../resources/images/albumplaceholder.png").as_slice(),
                );
            }

            self.add_album(&album_title, picture_handle, entry.path().to_path_buf());
        }
        dhat::ad_hoc_event(1);
        Ok(())
    }
    pub fn add_album(&mut self, title: &str, picture: impl Into<image::Handle>, file: PathBuf) {
        // If album already exists, append the file to it
        let existing_album = self
            .albums
            .par_iter_mut()
            .find_any(|album| album.title == title);
        if existing_album.is_some() {
            existing_album.unwrap().add_file(file);
            return;
        }
        // If not, add it to the list of albums
        self.albums
            .push(Album::new(title.to_string(), picture, vec![file]));
    }
}

#[derive(Debug, Clone)]
pub struct Album {
    pub title: String,
    pub picture: image::Handle,
    pub files: Vec<PathBuf>,
}
impl Album {
    pub fn new(title: String, picture: impl Into<image::Handle>, files: Vec<PathBuf>) -> Self {
        Album {
            title,
            picture: picture.into(),
            files,
        }
    }
    pub fn add_file(&mut self, new_file: PathBuf) {
        // If file is already in album, do not add it
        if self
            .files
            .par_iter()
            .find_any(|file| **file == new_file)
            .is_some()
        {
            return;
        }
        self.files.push(new_file);
    }
}

pub struct AlbumsPage {
    albums: AlbumsLibrary,
}

impl AlbumsPage {
    pub fn new(music_dir: path::PathBuf) -> Result<AlbumsPage, Box<dyn error::Error>> {
        let mut albums = AlbumsLibrary::default();
        albums.populate(music_dir.clone())?;

        Ok(AlbumsPage { albums })
    }
}

impl Page for AlbumsPage {
    fn update(&mut self, message: Message) -> Option<Box<dyn Page>> {
        if let Message::AlbumsPage(album_message) = message {
            match album_message {
                AlbumsPageMessage::ShowAlbum(album) => {
                    println!("{}", album.title)
                }
            }
        }
        None
    }
    fn view(&self) -> cosmic::Element<'_, Message> {
        elements_from_albums(&self.albums)
    }
}

fn elements_from_albums(albums: &AlbumsLibrary) -> Element<'static, Message> {
    let space = cosmic::theme::spacing().space_s;
    // let space_s = cosmic::theme::spacing().space_xxxs;
    let mut albums_grid: Vec<Element<Message>> = vec![];
    for album in albums.get_albums() {
        // let picture_handle: image::Handle;
        // if album.1.is_empty() {
        //     picture_handle = image::Handle::from_bytes(
        //         include_bytes!("../../resources/images/albumplaceholder.png").as_slice(),
        //     );
        // } else {
        //     picture_handle = image::Handle::from_bytes(album.1.first().unwrap().data().to_owned())
        // }
        static CARD_WIDTH: u16 = 150;
        let picture: Element<Message> = image(album.picture.clone()).width(CARD_WIDTH).into();
        let label = text(album.title.clone()).width(CARD_WIDTH);
        let album_card: Element<Message> =
            button::custom(column::with_capacity(2).push(picture).push(label))
                .on_press(Message::AlbumsPage(AlbumsPageMessage::ShowAlbum(
                    album.clone(),
                )))
                .height(200)
                .padding(space)
                .into();
        albums_grid.push(album_card);
    }

    // for _i in 0..albums.len() / 5 {
    //     albums_grid = albums_grid.insert_row();
    // }

    Element::from(scrollable(
        flex_row(albums_grid)
            .justify_content(JustifyContent::SpaceBetween)
            .spacing(space)
            .padding(space)
            .width(Length::Fill),
    ))
}
