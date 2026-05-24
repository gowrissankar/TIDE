mod app;
mod input;
mod life;
mod render;
mod screen;

use app::App;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.run()
}
