use app::App;
use color_eyre::eyre::Result;

mod app;
mod widget;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Setup terminal
    let mut terminal = ratatui::init();

    // Run app
    let mut app = App::default();
    let result = app.run(&mut terminal);

    // Restore terminal
    ratatui::restore();

    return result;
}
