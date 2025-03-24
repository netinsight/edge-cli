use clap::{ArgMatches, Command};

use crate::kubernetes;

use crate::edge::new_client;

pub(crate) fn subcommand() -> clap::Command {
    Command::new("node")
        .about("Show information about kubernetes nodes")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("List kubernetes nodes"))
}

pub(crate) fn run(subcmd: &ArgMatches) {
    match subcmd.subcommand() {
        Some(("list", _)) => kubernetes::list_nodes(new_client()),
        _ => unreachable!("subcommand_required prevents `None` or other options"),
    }
}
