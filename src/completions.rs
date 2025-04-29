use std::process;

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::cli;

pub(crate) fn subcommand() -> clap::Command {
    Command::new("completion")
        .about("Generate shell completion script")
        .arg(
            Arg::new("shell")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(clap_complete::aot::Shell)),
        )
}

fn print_completions<G: clap_complete::aot::Generator>(shell: G, cmd: &mut Command) {
    clap_complete::aot::generate(
        shell,
        cmd,
        cmd.get_name().to_string(),
        &mut std::io::stdout(),
    );
}

pub(crate) fn run(subcmd: &ArgMatches) {
    let shell = subcmd
        .get_one::<clap_complete::aot::Shell>("shell")
        .copied()
        .or_else(clap_complete::aot::Shell::from_env);
    let shell = match shell {
        None => {
            eprintln!("Failed to determine shell from environment, specify it manually with the --shell flag");
            process::exit(1);
        }
        Some(shell) => shell,
    };

    let mut cmd = cli::build();
    print_completions(shell, &mut cmd);
}
