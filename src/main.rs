mod appliance;
mod buildinfo;
mod cli;
mod colors;
mod completions;
mod edge;
mod group;
mod health;
mod input;
mod kubernetes;
mod node;
mod output;
mod output_list;
mod region;
mod settings;
mod tunnels;

use std::{env, process};

use edge::EdgeClient;

fn main() {
    let matches = cli::build().get_matches();

    match matches.subcommand() {
        Some(("input", subcmd)) => input::run(subcmd),
        Some(("output", subcmd)) => output::run(subcmd),
        Some(("output-list", subcmd)) => output_list::run(subcmd),
        Some(("appliance", subcmd)) => appliance::run(subcmd),
        Some(("group", subcmd)) => group::run(subcmd),
        Some(("region", subcmd)) => region::run(subcmd),
        Some(("node", subcmd)) => node::run(subcmd),
        Some(("tunnel", subcmd)) => tunnels::run(subcmd),
        Some(("settings", subcmd)) => settings::run(subcmd),
        Some(("completion", subcmd)) => completions::run(subcmd),
        Some(("health", subcmd)) => health::run(subcmd),
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
