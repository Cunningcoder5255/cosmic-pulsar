#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
use cosmic::app;
use cosmic_pulsar::app::App;

fn main() -> Result<(), cosmic::iced::Error> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    #[cfg(feature = "dhat-ad-hoc")]
    let _profiler = dhat::Profiler::new_ad_hoc();

    let settings = app::Settings::default().size(cosmic::iced::Size::new(3840.0, 2160.0)); //.debug(true);
    cosmic::app::run::<App>(settings, ())?;
    Result::Ok(())
}
