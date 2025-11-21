pub mod app;
pub mod events;
pub mod resources;
pub mod ui;

use crate::edge::new_client;
use anyhow::{Context, Result};
pub use app::App;
pub use events::run_event_loop;
use std::env;

pub fn run() -> Result<()> {
    // Validate environment variables before initializing terminal
    env::var("EDGE_URL").context("Missing environment variable: EDGE_URL")?;
    env::var("EDGE_PASSWORD").context("Missing environment variable: EDGE_PASSWORD")?;

    // Create authenticated client
    let client = new_client();

    // Create app state
    let app = App::new(client)?;

    // Run the event loop
    run_event_loop(app)?;

    Ok(())
}
