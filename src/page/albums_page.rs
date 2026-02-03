// use crate::app::App;
use crate::page::card_style;
use crate::player::PlayerMessage;
use crate::song_library::SongLibrary;
use std::collections::HashMap;
use std::collections::HashSet;
extern crate rayon;
use crate::song::Song;
use cosmic::iced_core::text::Wrapping;
use std::io::Write;
extern crate walkdir;
use crate::app::Message;
use crate::page::Page;
use cosmic;
use cosmic::Element;
// use cosmic::iced::Alignment;
use cosmic::iced::Length;
use cosmic::widget::*;
use derivative::Derivative;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::error;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum AlbumsPageMessage {
    ShowAlbum(String),
    Populate(Option<Song>),
    BackToAllAlbums,
    PopulateAlbumsLibrary,
}

pub struct AlbumsPage {
    albums_library: SongLibrary,
}

impl AlbumsPage {
    pub fn new(
        music_dir: &Path,
    ) -> Result<(AlbumsPage, cosmic::Task<cosmic::Action<Message>>), Box<dyn error::Error>> {
        let albums = SongLibrary::default();
        let populate_task = cosmic::Task::batch(SongLibrary::populate(music_dir.into()))
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
                AlbumsPageMessage::Populate(some_song) => {
                    some_song.map(|song| self.albums_library.add_song(song));
                }
            }
        }
        (cosmic::Task::none(), None)
    }
    fn view(&self) -> cosmic::Element<'_, Message> {
        if self.albums_library.show_album.is_none() {
            return elements_from_albums(&self.albums_library);
            eprintln!("Showing albums.");
        }
        elements_from_songs(
            self.albums_library.show_album.as_ref().unwrap(),
            &self.albums_library,
        )
    }
}

fn elements_from_songs<'a>(album_title: &str, library: &'a SongLibrary) -> Element<'a, Message> {
    // println!("Displaying...");
    let space = cosmic::theme::spacing().space_s;
    let mut songs_list: Vec<Element<Message>> = vec![];
    songs_list.push(
        button::text("Back")
            .on_press(Message::AlbumsPage(AlbumsPageMessage::BackToAllAlbums))
            .into(),
    );
    let album: Vec<Song> = library.get_album(album_title);

    album.clone().iter().enumerate().for_each(|(i, song)| {
        let button = button::custom(song.display())
            .on_press(Message::Player(PlayerMessage::PlaySongs(album.clone(), i)));
        songs_list.push(button.into());
    });
    // println!("Done");
    scrollable(
        column::with_children(songs_list)
            .spacing(space)
            .padding(space),
    )
    .into()
}

fn elements_from_albums(albums: &SongLibrary) -> Element<'static, Message> {
    let space = cosmic::theme::spacing().space_s;
    let space_s = cosmic::theme::spacing().space_xxs;
    let space_xs = cosmic::theme::spacing().space_xxxs;
    let mut albums_grid: Vec<Element<Message>> = vec![];
    for album in albums.get_albums() {
        static CARD_WIDTH: f32 = 100.0;
        let picture: Element<Message> = image(album.1[0].picture.clone())
            .border_radius([4.0; 4]) // Currently doesn't work with hardware rendering
            .into();
        let label = text(album.0.clone())
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
                album.0.clone(),
            )))
            .height(CARD_WIDTH * 1.5)
            .padding(space_s),
        )
        .style(card_style)
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
