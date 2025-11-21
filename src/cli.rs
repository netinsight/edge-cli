use crate::alarm;
use crate::appliance;
use crate::completions;
use crate::group;
use crate::group_list;
use crate::health;
use crate::input;
use crate::node;
use crate::output;
use crate::output_list;
use crate::region;
use crate::settings;
use crate::tunnels;

use clap::Command;

pub(crate) fn build() -> Command {
    let version_string: &'static str = Box::leak(
        format!(
            "{} {} {}",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            env!("BUILD_DATE")
        )
        .into_boxed_str(),
    );

    Command::new("edgectl")
        .about("Nimbra Edge CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .long_version(version_string)
        .subcommand_required(true)
        .subcommand(alarm::subcommand())
        .subcommand(input::subcommand())
        .subcommand(output::subcommand())
        .subcommand(output_list::subcommand())
        .subcommand(appliance::subcommand())
        .subcommand(group::subcommand())
        .subcommand(group_list::subcommand())
        .subcommand(region::subcommand())
        .subcommand(node::subcommand())
        .subcommand(tunnels::subcommand())
        .subcommand(settings::subcommand())
        .subcommand(completions::subcommand())
        .subcommand(health::subcommand())
        .subcommand(Command::new("build-info").about("Show build information for installation"))
        .subcommand(Command::new("open").about("Open interactive TUI"))
}
