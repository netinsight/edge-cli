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
                .subcommand(
                    Command::new("create")
                        .arg(
                            Arg::new("name")
                                .required(true)
                                .help("The name of the new input"),
                        )
                        .arg(
                            Arg::new("appliance")
                                .short('a')
                                .long("appliance")
                                .required(true)
                                .help("The appliance to create the input on"),
                        )
                        .arg(
                            Arg::new("mode")
                                .short('m')
                                .long("mode")
                                .required(true)
                                .value_parser(clap::builder::PossibleValuesParser::new([
                                    "rtp", "udp",
                                ]))
                                .help("The input mode"),
                        )
                        .arg(
                            Arg::new("interface")
                                .short('i')
                                .long("interface")
                                .required(true)
                                .help("The interface on the appliance to create the input on"),
                        )
                        .arg(
                            Arg::new("disable-thumbnails")
                                .long("disable-thumbnails")
                                .num_args(0)
                                .help("Disable thumbnailing"),
                        )
                        .arg(
                            Arg::new("port")
                                .short('p')
                                .long("port")
                                .value_parser(clap::value_parser!(u16).range(1..))
                                .action(clap::ArgAction::Set)
                                .required(false)
                                .help("The TCP or UDP port to listen to"),
                        )
                        .arg(
                            Arg::new("fec")
                                .long("fec")
                                .num_args(0)
                                .help("Enable FEC for RTP inputs"),
                        )
                        .arg(
                            Arg::new("multicast")
                                .long("multicast")
                                .help("Specify source multicast address for RTP and UDP inputs"),
                        ),
                )
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
            Some(("create", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("name is required");
                let appliance = args
                    .get_one::<String>("appliance")
                    .map(|s| s.as_str())
                    .expect("appliance is required");
                let port = args.get_one::<u16>("port");
                let interface = args
                    .get_one::<String>("interface")
                    .map(|s| s.as_str())
                    .expect("interface is required");
                let mode = args
                    .get_one::<String>("mode")
                    .map(|s| s.as_str())
                    .expect("mode is required");
                let disable_thumbnails = args.get_flag("disable-thumbnails");
                let multicast = args.get_one::<String>("multicast").map(|s| s.as_str());

                if port.is_some() && mode != "rtp" && mode != "udp" {
                    eprintln!("The port flag is not supported with mode {}", mode);
                    process::exit(1);
                }

                if args.get_flag("fec") && mode != "rtp" {
                    eprintln!("The fec flag is only supported with RTP inputs");
                    process::exit(1);
                }

                if multicast.is_some() && mode != "rtp" && mode != "udp" {
                    eprintln!("The multicast flag is not supported with mode {}", mode);
                    process::exit(1);
                }

                let mode = match mode {
                    "rtp" => {
                        let port = match port {
                            Some(p) => p,
                            None => {
                                eprintln!("Port is required for RTP inputs");
                                process::exit(1);
                            }
                        };
                        input::NewInputMode::Rtp(input::NewRtpInputMode {
                            port: *port,
                            fec: args.get_flag("fec"),
                            multicast_address: multicast.map(|s| s.to_owned()),
                        })
                    }
                    "udp" => {
                        let port = match port {
                            Some(p) => p,
                            None => {
                                eprintln!("Port is required for UDP inputs");
                                process::exit(1);
                            }
                        };
                        input::NewInputMode::Udp(input::NewUdpInputMode {
                            port: *port,
                            multicast_address: multicast.map(|s| s.to_owned()),
                        })
                    }
                    e => {
                        eprintln!("Invalid mode: {}", e);
                        process::exit(1);
                    }
                };

                input::create(
                    client,
                    input::NewInput {
                        name: name.to_owned(),
                        appliance: appliance.to_owned(),
                        interface: interface.to_owned(),
                        thumbnails: !disable_thumbnails,
                        mode,
                    },
                )
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
