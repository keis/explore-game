/// Views are interface bits that stand alone, they typically reference the global state of the
/// engine as opposed to that of some specific entity.
mod camp;
mod game_over;
mod menu;
mod selected;
mod shell;

pub use camp::CampView;
pub use game_over::GameOverView;
pub use menu::MenuView;
pub use selected::SelectedView;
pub use shell::ShellView;
