pub mod app;
pub mod events;
pub mod resources;
pub mod ui;

use crate::edge::new_client;
use anyhow::Result;
pub use app::App;
pub use events::run_event_loop;

pub fn run() -> Result<()> {
    let client = new_client();
    let app = App::new(client)?;
    run_event_loop(app)?;

    Ok(())
}
