use std::process;

use crate::edge::{new_client, EdgeClient};

use clap::{builder::PossibleValuesParser, Arg, ArgMatches, Command};

pub(crate) fn subcommand() -> clap::Command {
    Command::new("settings")
        .about("Manage global settings")
        .subcommand_required(false)
        .subcommand(
            Command::new("loglevel")
                .about("Set the Edge API log level")
                .arg(
                    Arg::new("loglevel")
                        .required(true)
                        .value_parser(PossibleValuesParser::new([
                            "fatal", "error", "warn", "info", "debug", "trace",
                        ])),
                ),
        )
}

pub(crate) fn run(subcmd: &ArgMatches) {
    match subcmd.subcommand() {
        Some(("loglevel", v)) => set_log_level(
            new_client(),
            match v.get_one::<String>("loglevel").map(|l| l.as_str()) {
                Some("fatal") => crate::edge::LogLevel::Fatal,
                Some("error") => crate::edge::LogLevel::Error,
                Some("warn") => crate::edge::LogLevel::Warn,
                Some("info") => crate::edge::LogLevel::Info,
                Some("debug") => crate::edge::LogLevel::Debug,
                Some("trace") => crate::edge::LogLevel::Trace,
                _ => unreachable!("clap ensures all values are covered"),
            },
        ),
        None => list(new_client()),
        _ => unreachable!("clap prevents  other options"),
    }
}

fn list(client: EdgeClient) {
    let settings = client
        .global_settings()
        .expect("Failed to fetch global settingst");
    eprintln!("Log level:   {:?}", settings.log_level);
}

fn set_log_level(client: EdgeClient, level: crate::edge::LogLevel) {
    // As the API lacks a PATCH method we fetch and replace the relevant values intead
    let mut settings = match client.global_settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to fetch settings for update: {}", e);
            process::exit(1);
        }
    };
    settings.log_level = level;

    if let Err(e) = client.set_global_settings(settings) {
        eprintln!("Failed to set log level: {}", e);
        process::exit(1);
    }
}
