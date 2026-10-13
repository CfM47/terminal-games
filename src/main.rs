mod app;
mod audio;
mod game;
mod games;
mod layout;
mod menu;
mod scores;
mod session;

use app::App;

fn main() -> std::io::Result<()> {
    let app = App::new(games::CATALOG);
    ratatui::run(|terminal| app.run(terminal))
}
