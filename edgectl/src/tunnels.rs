use tabled::{builder::Builder, settings::Style};

use crate::edge::{new_client, EdgeClient};

use clap::{ArgMatches, Command};

pub(crate) fn subcommand() -> clap::Command {
    Command::new("tunnel")
        .about("Show information about tunnels")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("List tunnels"))
}

pub(crate) fn run(subcmd: &ArgMatches) {
    match subcmd.subcommand() {
        Some(("list", _)) => list(new_client()),
        _ => unreachable!("subcommand_required prevents `None` or other options"),
    }
}

fn list(client: EdgeClient) {
    let tunnels = client.list_tunnels().expect("Failed to fetch tunnel list");

    let mut builder = Builder::default();

    builder.push_record(["ID", "Type", "Client", "Server", "Inputs"]);
    for tunnel in tunnels {
        builder.push_record([
            tunnel.id.to_string(),
            tunnel.r#type.to_string(),
            tunnel.client_name,
            tunnel.server_name,
            tunnel.inputs.len().to_string(),
        ])
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}
