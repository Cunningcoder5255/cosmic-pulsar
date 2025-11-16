// use crate::app::App;
extern crate rayon;
extern crate walkdir;
use crate::app::Message;
use crate::page::Page;
use cosmic;
use cosmic::Element;
use derivative::Derivative;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
// use cosmic::iced::Alignment;
use cosmic::iced::Length;
use cosmic::widget::*;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use std::error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum AlbumsPageMessage {
    ShowAlbum(String),
    BackToAllAlbums,
}

struct AlbumsLibrary {
    albums: BTreeSet<Album>,
    show_album: Option<String>,
}
impl AlbumsLibrary {
    pub fn default() -> Self {
        AlbumsLibrary {
            albums: vec![].into_iter().collect(),
            show_album: None,
        }
    }
    pub fn get_albums(&self) -> &BTreeSet<Album> {
        &self.albums
    }
    pub fn populate(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn error::Error>> {
        let mut paths: Vec<PathBuf> = vec![];

        // Go over the entries in the directory
        // Should make parallel at some point but im too retarded for that
        for entry in WalkDir::new(path)
            .contents_first(true)
            .follow_links(true)
            .into_iter()
            .filter(|e| {
                if let Ok(e) = e {
                    return !e.file_type().is_dir();
                } else {
                    return false;
                };
            })
        {
            let entry = entry?; // Return errors with entry

            paths.push(entry.into_path());
        }

        // Does this even do anything? Mutex might lock it so that you can only perform one operation at a time anyways
        let self_ref = Arc::new(Mutex::new(self));
        paths.par_iter().for_each(|path| {
            println!("Adding file: {:#?}", path);
            self_ref.lock().unwrap().add_file(&path)
        });
        Ok(())
    }
    pub fn add_file(&mut self, file: &Path) {
        // Create a song from the given file path
        let Ok(song) = Song::from_path(&file) else {
            return;
        };
        let Some(album_title) = song.album_title.as_ref() else {
            return;
        };
        // Do not use `song` yet because it would force us to clone later and is not relevant for comparisons
        let mut album = Album::new(album_title, vec![].into_iter().collect());
        // If the album is already in the set, we add the given song to it.
        // Album Eq and Hash is not effected by the list of songs, so we can find and compare albums that have more songs than the file we've been given
        if self.albums.contains(&album) {
            album = self.albums.take(&album).unwrap();
            album.add_song(song);
            self.albums.insert(album);
        } else {
            album.add_song(song);
            self.albums.insert(album);
        }
    }
}

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
    fn new(
        title: String,
        artist: Option<String>,
        album_title: Option<String>,
        genre: Option<String>,
        year: Option<u32>,
        picture: image::Handle,
        path: &Path,
        index: Option<u32>,
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
        }
    }
    fn from_path(path: &Path) -> Result<Self, lofty::error::LoftyError> {
        let lofty_file = lofty::read_from_path(path)?;
        let file_tag = lofty_file
            .primary_tag()
            .ok_or(lofty::error::LoftyError::new(
                lofty::error::ErrorKind::FakeTag,
            ))?;
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
            .ok_or(lofty::error::LoftyError::new(
                lofty::error::ErrorKind::TextDecode("Could not convert file name to rust string."),
            ))?;
        // Set title either to the title tag or the file name
        let title = file_tag.title().unwrap_or_else(|| file_name.into());
        let album_title = file_tag.album().map(|title| title.to_string());
        let picture = file_tag.pictures().to_vec().pop();
        let picture_handle: image::Handle;
        if let Some(picture) = picture {
            picture_handle = image::Handle::from_bytes(picture.into_data());
        } else {
            picture_handle = image::Handle::from_bytes(
                include_bytes!("../../resources/images/albumplaceholder.png").as_slice(),
            );
        }
        let index = file_tag.track();
        let artist = file_tag.artist().map(|artist| artist.to_string());
        let genre = file_tag.genre().map(|genre| genre.to_string());
        let year = file_tag.year();

        Ok(Self::new(
            title.to_string(),
            artist,
            album_title,
            genre,
            year,
            picture_handle,
            path,
            index,
        ))
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Album {
    pub title: String,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        PartialOrd = "ignore",
        Ord = "ignore"
    )]
    pub placeholder_image: image::Handle,
    #[derivative(Hash = "ignore", PartialEq = "ignore", Ord = "ignore")]
    pub songs: BTreeSet<Song>,
}
impl Album {
    pub fn new(title: &str, songs: BTreeSet<Song>) -> Album {
        let placeholder_image = image::Handle::from_bytes(
            include_bytes!("../../resources/images/albumplaceholder.png").as_slice(),
        );
        Album {
            title: title.to_string(),
            placeholder_image,
            songs,
        }
    }
    pub fn from_song(song: Song) -> Album {
        let title = song.title.clone();

        Self::new(&title, vec![song].into_iter().collect())
    }
    pub fn add_song(&mut self, song: Song) {
        self.songs.insert(song);
    }
    pub fn get_songs(&self) -> &BTreeSet<Song> {
        &self.songs
    }
    /// Gets the album picture from the first song in the album
    pub fn get_picture(&self) -> &image::Handle {
        let picture: &image::Handle;
        if let Some(song) = &self.songs.first() {
            picture = &song.picture;
        } else {
            picture = &self.placeholder_image;
        }
        picture
    }
}

pub struct AlbumsPage {
    albums: AlbumsLibrary,
}

impl AlbumsPage {
    pub fn new(music_dir: &Path) -> Result<AlbumsPage, Box<dyn error::Error>> {
        let mut albums = AlbumsLibrary::default();
        albums.populate(music_dir)?;

        Ok(AlbumsPage { albums })
    }
}

impl Page for AlbumsPage {
    fn update(&mut self, message: Message) -> Option<Box<dyn Page>> {
        if let Message::AlbumsPage(album_message) = message {
            match album_message {
                AlbumsPageMessage::ShowAlbum(title) => {
                    self.albums.show_album = Some(title);
                }
                AlbumsPageMessage::BackToAllAlbums => {
                    self.albums.show_album = None;
                }
            }
        }
        None
    }
    fn view(&self) -> cosmic::Element<'_, Message> {
        if self.albums.show_album.is_none() {
            return elements_from_albums(&self.albums);
        }
        elements_from_songs(self.albums.show_album.as_ref().unwrap(), &self.albums)
    }
}

fn elements_from_songs(album: &str, library: &AlbumsLibrary) -> Element<'static, Message> {
    let mut songs_list: Vec<Element<Message>> = vec![];
    let Some(album) = library
        .get_albums()
        .par_iter()
        .find_any(|library_album| library_album.title == album)
    else {
        return text("No album.").into();
    };

    for song in album.get_songs() {}
    button::text("Back")
        .on_press(Message::AlbumsPage(AlbumsPageMessage::BackToAllAlbums))
        .into()
}

fn elements_from_albums(albums: &AlbumsLibrary) -> Element<'static, Message> {
    let space = cosmic::theme::spacing().space_s;
    // let space_s = cosmic::theme::spacing().space_xxxs;
    let mut albums_grid: Vec<Element<Message>> = vec![];
    for album in albums.get_albums() {
        static CARD_WIDTH: u16 = 150;
        let picture: Element<Message> = image(album.get_picture().clone()).width(CARD_WIDTH).into();
        let label = text(album.title.clone()).width(CARD_WIDTH);
        let album_card: Element<Message> =
            button::custom(column::with_capacity(2).push(picture).push(label))
                .on_press(Message::AlbumsPage(AlbumsPageMessage::ShowAlbum(
                    album.title.clone(),
                )))
                .height(200)
                .padding(space)
                .into();
        albums_grid.push(album_card);
    }

    Element::from(scrollable(
        flex_row(albums_grid)
            .justify_content(JustifyContent::SpaceBetween)
            .spacing(space)
            .padding(space)
            .width(Length::Fill),
    ))
}
