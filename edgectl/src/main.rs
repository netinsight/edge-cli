mod edge;
mod group;
mod input;

use std::{env, process};

use clap::{Arg, Command};

use edge::EdgeClient;

fn main() {
    let matches = Command::new("edgectl")
        .about("Nimbra Edge CLI")
        .subcommand_required(true)
        .subcommand(
            Command::new("input")
                .about("Manage inputs")
                .subcommand_required(true)
                .subcommand(
                    Command::new("list").arg(
                        Arg::new("output")
                            .long("output")
                            .short('o')
                            .value_parser(["short", "wide"])
                            .default_value("short")
                            .help("Change the output format"),
                    ),
                )
                .subcommand(
                    Command::new("show").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The input name to show details for"),
                    ),
                )
                .subcommand(Command::new("create"))
                .subcommand(
                    Command::new("delete").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The name of the inputs to remove"),
                    ),
                ),
        )
        .subcommand(
            Command::new("output")
                .about("Manage outputs")
                .subcommand_required(true)
                .subcommand(Command::new("list"))
                .subcommand(Command::new("show"))
                .subcommand(Command::new("create"))
                .subcommand(Command::new("delete")),
        )
        .subcommand(
            Command::new("appliance")
                .about("Manage appliances")
                .subcommand_required(true)
                .subcommand(Command::new("list"))
                .subcommand(Command::new("inputs"))
                .subcommand(Command::new("outputs"))
                .subcommand(Command::new("delete"))
                .subcommand(Command::new("config")),
        )
        .subcommand(
            Command::new("group")
                .about("Manage groups")
                .subcommand_required(true)
                .subcommand(Command::new("list")),
        )
        .subcommand(
            Command::new("region")
                .about("Manage regions")
                .subcommand(Command::new("list"))
                .subcommand(Command::new("delete")),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("input", subcmd)) => match subcmd.subcommand() {
            Some(("list", args)) => {
                let client = new_client();
                match args.get_one::<String>("output").map(|s| s.as_str()) {
                    Some("wide") => input::list_wide(client),
                    _ => input::list(client),
                };
            }
            Some(("show", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("input name should not be None");

                input::show(client, name);
            }
            Some(("delete", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("name should not be None");

                input::delete(client, name).unwrap();
            }
            Some((cmd, _)) => {
                eprintln!("Command input {cmd} is not yet implemented");
                process::exit(1);
            }
            None => unreachable!("subcommand_required prevents `None`"),
        },
        Some(("group", subcmd)) => match subcmd.subcommand() {
            Some(("list", _)) | None => {
                let client = new_client();
                group::list(client)
            }
            _ => unreachable!("subcommand_required prevents `None` or other options"),
        },
        Some((cmd, _)) => {
            eprintln!("Command {cmd} is not yet implemented");
            process::exit(1);
        }
        None => unreachable!("subcommand_required prevents `None`"),
    }
}

fn new_client() -> EdgeClient {
    let client = EdgeClient::with_url(
        env::var("EDGE_URL")
            .expect("missing environment variable: EDGE_URL")
            .as_ref(),
    );
    if let Err(e) = client.login(
        "admin".to_owned(),
        env::var("EDGE_PASSWORD").expect("missing environment variable: EDGE_PASSWORD"),
    ) {
        eprintln!("Failed to authorize against the API: {}", e);
        process::exit(1);
    }

    client
}
