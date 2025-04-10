mod appliance;
mod buildinfo;
mod edge;
mod group;
mod input;
mod kubernetes;
mod node;
mod output;
mod region;
mod settings;
mod tunnels;

use std::{env, process};

use clap::Command;

use edge::EdgeClient;

fn main() {
    let matches = Command::new("edgectl")
        .about("Nimbra Edge CLI")
        .subcommand_required(true)
        .subcommand(input::subcommand())
        .subcommand(output::subcommand())
        .subcommand(appliance::subcommand())
        .subcommand(group::subcommand())
        .subcommand(region::subcommand())
        .subcommand(node::subcommand())
        .subcommand(tunnels::subcommand())
        .subcommand(settings::subcommand())
        .subcommand(Command::new("build-info").about("Show build information for installation"))
        .get_matches();

    match matches.subcommand() {
        Some(("input", subcmd)) => input::run(subcmd),
        Some(("output", subcmd)) => output::run(subcmd),
        Some(("appliance", subcmd)) => appliance::run(subcmd),
        Some(("group", subcmd)) => group::run(subcmd),
        Some(("region", subcmd)) => region::run(subcmd),
        Some(("node", subcmd)) => node::run(subcmd),
        Some(("tunnel", subcmd)) => tunnels::run(subcmd),
        Some(("settings", subcmd)) => settings::run(subcmd),
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
