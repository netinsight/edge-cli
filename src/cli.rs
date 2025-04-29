use crate::appliance;
use crate::completions;
use crate::group;
use crate::input;
use crate::node;
use crate::output;
use crate::region;
use crate::settings;
use crate::tunnels;

use clap::Command;

pub(crate) fn build() -> Command {
    Command::new("edgectl")
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
        .subcommand(completions::subcommand())
        .subcommand(Command::new("build-info").about("Show build information for installation"))
}
