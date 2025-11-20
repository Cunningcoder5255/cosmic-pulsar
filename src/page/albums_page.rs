// use crate::app::App;
extern crate rayon;
use crate::song::Song;
use cosmic::iced_core::text::Wrapping;
use std::io::Write;
extern crate walkdir;
use crate::app::Message;
use crate::page::Page;
use cosmic;
use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::iced::Length;
use cosmic::widget::*;
use derivative::Derivative;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::error;
use std::path::{Path, PathBuf};
// use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum AlbumsPageMessage {
    ShowAlbum(String),
    Populate(Option<Album>),
    BackToAllAlbums,
    PopulateAlbumsLibrary,
}

#[derive(Debug, Clone)]
pub struct AlbumsLibrary {
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
    /// Inspects the directory given and returns a task to create an album out of each file
    // Should run async on startup in the future
    pub fn populate(path: PathBuf) -> Vec<cosmic::Task<Option<Album>>> {
        let mut lock = std::io::stderr().lock();
        let _ = writeln!(lock, "{:#?}", path);
        let mut paths: Vec<PathBuf> = vec![];

        for entry in WalkDir::new(path)
            .contents_first(true)
            .follow_links(true)
            .into_iter()
            .filter(|e| {
                if let Ok(e) = e {
                    let Ok(metadata) = Path::metadata(e.path()) else {
                        return false;
                    };
                    return metadata.is_file();
                } else {
                    return false;
                };
            })
        {
            let _ = writeln!(lock, "Inspecting directory/file: {:#?}", entry);
            let Ok(entry) = entry else { continue }; // Skip bad paths

            paths.push(entry.into_path());
        }

        let mut tasks: Vec<cosmic::Task<Option<Album>>> = vec![];

        for path in paths {
            // Skip invalid songs
            // let Ok(song) = Song::from_path(path) else {
            //     continue;
            // };
            let _ = writeln!(lock, "Pushing task for: {:#?}", path);
            tasks.push(
                cosmic::Task::perform(Song::from_path(path), |song| song)
                    .and_then(|song| cosmic::Task::perform(Album::from_song(song), |album| album)),
            );
            // tasks.push(cosmic::Task::perform(Album::from_song(song), |album| album))
        }

        tasks
    }
    pub async fn add_file(&mut self, file: &Path) {
        // Create a song from the given file path
        let Ok(song) = Song::from_path(file.into()).await else {
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
    pub fn add_album(&mut self, album: Album) {
        if self.albums.contains(&album) {
            let mut old_album = self.albums.take(&album).unwrap();
            old_album = old_album.union(album);
            self.albums.insert(old_album);
        } else {
            self.albums.insert(album);
        }
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
    pub fn union(mut self, mut other_album: Album) -> Self {
        self.songs.append(&mut other_album.songs);
        self
    }
    pub async fn from_song(song: Song) -> Option<Album> {
        println!("Creating album from song: {:#?}", song);
        let Some(title) = song.album_title.clone() else {
            return None;
        };
        Some(Self::new(&title, vec![song].into_iter().collect()))
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
    albums_library: AlbumsLibrary,
}

impl AlbumsPage {
    pub fn new(
        music_dir: &Path,
    ) -> Result<(AlbumsPage, cosmic::Task<cosmic::Action<Message>>), Box<dyn error::Error>> {
        let albums = AlbumsLibrary::default();
        let populate_task = cosmic::Task::batch(AlbumsLibrary::populate(music_dir.into()))
            .map(|o| cosmic::Action::App(Message::AlbumsPage(AlbumsPageMessage::Populate(o))));

        Ok((
            AlbumsPage {
                albums_library: albums,
            },
            populate_task,
        ))
    }
}

impl Page for AlbumsPage {
    fn update(
        &mut self,
        message: Message,
    ) -> (cosmic::Task<cosmic::Action<Message>>, Option<Box<dyn Page>>) {
        if let Message::AlbumsPage(album_message) = message {
            match album_message {
                AlbumsPageMessage::ShowAlbum(title) => {
                    self.albums_library.show_album = Some(title);
                }
                AlbumsPageMessage::BackToAllAlbums => {
                    self.albums_library.show_album = None;
                }
                AlbumsPageMessage::PopulateAlbumsLibrary => {}
                AlbumsPageMessage::Populate(some_album) => {
                    some_album.map(|album| self.albums_library.add_album(album));
                }
            }
        }
        (cosmic::Task::none(), None)
    }
    fn view(&self) -> cosmic::Element<'_, Message> {
        if self.albums_library.show_album.is_none() {
            return elements_from_albums(&self.albums_library);
        }
        elements_from_songs(
            self.albums_library.show_album.as_ref().unwrap(),
            &self.albums_library,
        )
    }
}

fn elements_from_songs(album: &str, library: &AlbumsLibrary) -> Element<'static, Message> {
    println!("Displaying...");
    let space = cosmic::theme::spacing().space_s;
    let mut songs_list: Vec<Element<Message>> = vec![];
    songs_list.push(
        button::text("Back")
            .on_press(Message::AlbumsPage(AlbumsPageMessage::BackToAllAlbums))
            .into(),
    );
    let Some(album) = library
        .get_albums()
        .par_iter()
        .find_any(|library_album| library_album.title == album)
    else {
        return text("Could not find album").into();
    };

    for song in album.get_songs() {
        println!("Displaying {:#?}", song);
        const HEIGHT: u16 = 100;
        let picture = image(song.picture.clone());
        let name = text(song.title.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Center);
        let index = text(song.index.map(|i| i.to_string()).unwrap_or_else(|| "".to_string()))
            .align_y(Alignment::Center)
            .height(Length::Fill);
        let container = container(
            row::with_capacity::<Message>(3)
                .push(picture)
                .push(name)
                .push(index)
                .spacing(space),
        )
        .height(HEIGHT);
        songs_list.push(container.into());
    }
    println!("Done");
    scrollable(
        column::with_children(songs_list)
            .spacing(space)
            .padding(space),
    )
    .into()
}

fn elements_from_albums(albums: &AlbumsLibrary) -> Element<'static, Message> {
    let space = cosmic::theme::spacing().space_s;
    let space_s = cosmic::theme::spacing().space_xxs;
    let space_xs = cosmic::theme::spacing().space_xxxs;
    let mut albums_grid: Vec<Element<Message>> = vec![];
    for album in albums.get_albums() {
        static CARD_WIDTH: f32 = 100.0;
        let picture: Element<Message> = image(album.get_picture().clone())
            .border_radius([4.0; 4]) // Currently doesn't work
            .into();
        let label = text(album.title.clone())
            .center()
            .wrapping(Wrapping::WordOrGlyph);
        let album_card: Element<Message> = container(
            button::custom(
                column::with_capacity(2)
                    .push(picture)
                    .push(label)
                    .spacing(space_xs),
            )
            .on_press(Message::AlbumsPage(AlbumsPageMessage::ShowAlbum(
                album.title.clone(),
            )))
            .height(CARD_WIDTH * 1.5)
            .padding(space_s),
        )
        .max_width(CARD_WIDTH)
        .into();
        albums_grid.push(album_card);
    }

    Element::from(
        container(scrollable(
            container(
                flex_row(albums_grid)
                    .justify_content(JustifyContent::SpaceEvenly)
                    .spacing(space)
                    // .padding(space)
                    .width(Length::Fill),
            )
            .padding(space),
        )), // .padding(space),
    )
}
