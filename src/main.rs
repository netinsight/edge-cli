mod alarm;
mod appliance;
mod buildinfo;
mod cli;
mod colors;
mod completions;
mod config;
mod context;
mod edge;
mod group;
mod group_list;
mod health;
mod input;
mod kubernetes;
mod login;
mod node;
mod output;
mod output_list;
mod region;
mod settings;
mod token;
mod tui;
mod tunnels;

use std::{env, process};

use edge::EdgeClient;

fn main() {
    let matches = cli::build().get_matches();

    match matches.subcommand() {
        Some(("alarm", subcmd)) => alarm::run(subcmd),
        Some(("input", subcmd)) => input::run(subcmd),
        Some(("output", subcmd)) => output::run(subcmd),
        Some(("output-list", subcmd)) => output_list::run(subcmd),
        Some(("appliance", subcmd)) => appliance::run(subcmd),
        Some(("group", subcmd)) => group::run(subcmd),
        Some(("group-list", subcmd)) => group_list::run(subcmd),
        Some(("region", subcmd)) => region::run(subcmd),
        Some(("node", subcmd)) => node::run(subcmd),
        Some(("tunnel", subcmd)) => tunnels::run(subcmd),
        Some(("settings", subcmd)) => settings::run(subcmd),
        Some(("completion", subcmd)) => completions::run(subcmd),
        Some(("health", subcmd)) => health::run(subcmd),
        Some(("login", subcmd)) => login::run(subcmd),
        Some(("token", subcmd)) => token::run(subcmd),
        Some(("context", subcmd)) => context::run(subcmd),
        Some(("tui", _)) => {
            if let Err(e) = tui::run() {
                eprintln!("Error running TUI: {}", e);
                process::exit(1);
            }
        }
        Some(("build-info", _)) => {
            let client = EdgeClient::with_url(
                env::var("EDGE_URL")
                    .expect("missing environment variable: EDGE_URL")
                    .as_ref(),
            );
            buildinfo::show(client)
        }
        Some((cmd, _)) => {
            eprintln!("Command {cmd} is not yet implemented");
            process::exit(1);
        }
        None => unreachable!("subcommand_required prevents `None`"),
    }
}
