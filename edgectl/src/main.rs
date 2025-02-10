mod appliance;
mod buildinfo;
mod edge;
mod group;
mod input;
mod output;
mod region;

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
                                    "rtp",
                                    "udp",
                                    "sdi",
                                    "generator",
                                ]))
                                .help("The input mode"),
                        )
                        .arg(
                            Arg::new("interface")
                                .short('i')
                                .long("interface")
                                .required(false)
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
                        )
                        .arg(
                            Arg::new("bitrate")
                                .long("bitrate")
                                .num_args(1)
                                .value_parser(|val: &str| -> Result<input::Bitrate, String> {
                                    if val == "vbr" {
                                        Ok(input::Bitrate::Vbr)
                                    } else {
                                        parse_bitrate(val).map(input::Bitrate::Cbr)
                                    }
                                })
                                .help("Set bitrate for generator"),
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
                            .help("The output name to show details for"),
                    ),
                )
                .subcommand(
                    Command::new("create")
                        .arg(
                            Arg::new("name")
                                .required(true)
                                .help("The name of the new output"),
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
                                    "rtp", "udp", "sdi",
                                ]))
                                .help("The input mode"),
                        )
                        .arg(
                            Arg::new("interface")
                                .long("interface")
                                .required(true)
                                .help("The interface on the appliance to create the input on"),
                        )
                        .arg(
                            Arg::new("input")
                                .long("input")
                                .required(true)
                                .help("The input to send to the output"),
                        )
                        .arg(
                            Arg::new("destination")
                                .short('d')
                                .long("dest")
                                .required(false)
                                .help("The destination to send the output to in format ip:port, e.g. 198.51.100.12:4000"),
                        )
                        .arg(
                            Arg::new("fec")
                                .long("fec")
                                .value_parser(["1D", "2D"])
                                .required(false)
                                .help("Enable FEC for RTP outputs"),
                        )
                        .arg(
                            Arg::new("fec-rows")
                                .long("fec-rows")
                                .value_parser(clap::value_parser!(u8).range(4..20))
                                .required(false)
                                .help("FEC rows"),
                        )
                        .arg(
                            Arg::new("fec-cols")
                                .long("fec-cols")
                                .value_parser(clap::value_parser!(u8).range(1..20))
                                .required(false)
                                .help("FEC columns"),
                        ),
                )
                .subcommand(
                    Command::new("delete").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The name of the outputs to remove"),
                    ),
                ),
        )
        .subcommand(
            Command::new("appliance")
                .about("Manage appliances")
                .subcommand_required(true)
                .subcommand(Command::new("list"))
                .subcommand(
                    Command::new("show").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The appliance name to show details for"),
                    ),
                )
                .subcommand(Command::new("inputs").about("List appliance inputs").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The appliance name to show details for"),
                    ),
                )
                .subcommand(Command::new("outputs").about("List appliance outputs").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The appliance name to show details for"),
                    ),
                )
                .subcommand(
                    Command::new("delete").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The name of the appliances to delete")
                            .num_args(1..),
                    ),
                )
                .subcommand(
                    Command::new("config").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The name of the appliance"),
                    ),
                )
                .subcommand(
                    Command::new("restart").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The name of the appliance"),
                    ),
                )
        )
        .subcommand(
            Command::new("group")
                .about("Manage groups")
                .subcommand_required(true)
                .subcommand(Command::new("list"))
                .subcommand(
                    Command::new("show").arg(
                        Arg::new("name")
                            .required(true)
                            .help("The group name to show details for"),
                    ),
                )
                .subcommand(
                    Command::new("create")
                        .arg(Arg::new("name").required(true).help("The group name")),
                )
                .subcommand(
                    Command::new("delete")
                        .arg(Arg::new("name").required(true).help("The group name")),
                )
                .subcommand(
                    Command::new("core-secret")
                        .arg(Arg::new("name").required(true).help("The group name")),
                ),
        )
        .subcommand(
            Command::new("region")
                .about("Manage regions")
                .subcommand(Command::new("list").about("List regions"))
                .subcommand(Command::new("create").arg(
                        Arg::new("name")
                        .required(true)
                        .help("The name of the region to create")
                ))
                .subcommand(Command::new("delete").arg(
                        Arg::new("name")
                        .required(true)
                        .help("The name of the region to create")
                )),
        )
        .subcommand(
            Command::new("build-info")
                .about("Show build information for installation")
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
                let interface = args.get_one::<String>("interface").map(|s| s.as_str());
                let mode = args
                    .get_one::<String>("mode")
                    .map(|s| s.as_str())
                    .expect("mode is required");
                let disable_thumbnails = args.get_flag("disable-thumbnails");
                let multicast = args.get_one::<String>("multicast").map(|s| s.as_str());
                let bitrate = args.get_one::<input::Bitrate>("bitrate");

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

                if bitrate.is_some() && mode != "generator" {
                    eprintln!("Bitrate is only supported for generator inputs");
                    process::exit(1);
                }

                let interface = {
                    if mode == "generator" {
                        if interface.is_some() {
                            eprintln!("Cannot specify interface for generator input");
                            process::exit(1)
                        }
                        "lo"
                    } else {
                        interface.expect("interface is required")
                    }
                };

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
                    "sdi" => input::NewInputMode::Sdi(input::NewSdiInputMode {}),
                    "generator" => input::NewInputMode::Generator(input::NewGeneratorInputMode {
                        bitrate: bitrate.unwrap_or(&input::Bitrate::Vbr).clone(),
                    }),
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
        Some(("output", subcmd)) => match subcmd.subcommand() {
            Some(("list", args)) => {
                let client = new_client();
                match args.get_one::<String>("output").map(|s| s.as_str()) {
                    Some("wide") => output::list_wide(client),
                    _ => output::list(client),
                };
            }
            Some(("show", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("output name should not be None");

                output::show(client, name);
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
                let mode = args
                    .get_one::<String>("mode")
                    .map(|s| s.as_str())
                    .expect("mode is required");
                let dest = args.get_one::<String>("destination").map(|s| s.as_str());
                let interface = args
                    .get_one::<String>("interface")
                    .map(|s| s.as_str())
                    .expect("interface is required");
                let input = args
                    .get_one::<String>("input")
                    .map(|s| s.as_str())
                    .expect("input is required");

                if args.get_one::<String>("fec").is_some() && mode != "rtp" {
                    eprintln!("The --fec argument is only supported for --mode rtp");
                    process::exit(1);
                }

                let mode = match mode {
                    "rtp" => {
                        let dest = match dest {
                            Some(d) => d,
                            None => {
                                eprintln!("Dest is required for UDP outputs");
                                process::exit(1);
                            }
                        };
                        let address = dest.split(':').next().expect("dest address is missing");
                        let port = dest
                            .split(':')
                            .last()
                            .expect("Port number is required for --dest")
                            .parse::<u16>()
                            .expect("port needs to be a number between 0 and 65535");

                        let fec = args.get_one::<String>("fec").map(|fec| {
                            match (args.get_one::<u8>("fec-rows"),args.get_one::<u8>("fec-cols")) {
                                (Some(rows), Some(cols)) => output::Fec {
                                    mode: match fec.as_ref() {
                                        "1D" => output::FecMode::OneD,
                                        "2D" => output::FecMode::TwoD,
                                        // clap ensures only 1D or 2D are possible values
                                        _ => panic!("Invalid FEC mode. This is bug"),
                                    },
                                    rows: *rows,
                                    cols: *cols,
                                },
                                _ =>  {
                                    eprintln!("The --fec argument requires the --fec-rows and --fec-cols arguments");
                                    process::exit(1);
                                }
                            }
                        });

                        output::NewOutputMode::Rtp(output::NewRtpOutputMode {
                            address: address.to_owned(),
                            port,
                            fec,
                        })
                    }
                    "udp" => {
                        let dest = match dest {
                            Some(d) => d,
                            None => {
                                eprintln!("Dest is required for UDP outputs");
                                process::exit(1);
                            }
                        };
                        let address = dest.split(':').next().expect("dest address is missing");
                        let port = dest
                            .split(':')
                            .last()
                            .expect("Port number is required for --dest")
                            .parse::<u16>()
                            .expect("port needs to be a number between 0 and 65535");
                        output::NewOutputMode::Udp(output::NewUdpOutputMode {
                            address: address.to_owned(),
                            port,
                        })
                    }
                    e => {
                        eprintln!("Invalid mode: {}", e);
                        process::exit(1);
                    }
                };

                output::create(
                    client,
                    output::NewOutput {
                        name: name.to_owned(),
                        appliance: appliance.to_owned(),
                        interface: interface.to_owned(),
                        input: input.to_owned(),
                        mode,
                    },
                )
            }
            Some(("delete", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("output should not be None");

                if let Err(e) = output::delete(client, name) {
                    eprintln!("Failed to delete output {}: {}", name, e);
                    process::exit(1);
                }
            }
            Some((cmd, _)) => {
                eprintln!("Command output {cmd} is not yet implemented");
                process::exit(1);
            }
            None => unreachable!("subcommand_required prevents `None`"),
        },
        Some(("appliance", subcmd)) => match subcmd.subcommand() {
            Some(("list", _)) | None => {
                let client = new_client();
                appliance::list(client)
            }
            Some(("show", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Appliance name is mandatory");
                appliance::show(client, name)
            }
            Some(("delete", args)) => {
                let client = new_client();
                let mut failed = false;
                for name in args
                    .get_many::<String>("name")
                    .expect("Appliance name is mandatory")
                {
                    if let Err(e) = appliance::delete(&client, name) {
                        eprintln!("Failed to delete appliance {}: {}", name, e);
                        failed = true;
                    }
                }
                if failed {
                    process::exit(1);
                }
            }
            Some(("inputs", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Appliance name is mandatory");
                appliance::inputs(client, name)
            }
            Some(("outputs", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Appliance name is mandatory");
                appliance::outputs(client, name)
            }
            Some(("config", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Appliance name is mandatory");
                appliance::config(client, name)
            }
            Some(("restart", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Appliance name is mandatory");
                appliance::restart(client, name)
            }
            _ => unreachable!("subcommand_required prevents `None` or other options"),
        },
        Some(("group", subcmd)) => match subcmd.subcommand() {
            Some(("list", _)) | None => {
                let client = new_client();
                group::list(client)
            }
            Some(("show", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Group name is mandatory");
                group::show(client, name)
            }
            Some(("core-secret", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Group name is mandatory");
                group::core_secret(client, name)
            }
            Some(("create", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Group name is mandatory");
                group::create(client, name)
            }
            Some(("delete", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Group name is mandatory");
                group::delete(client, name)
            }
            _ => unreachable!("subcommand_required prevents `None` or other options"),
        },
        Some(("region", subcmd)) => match subcmd.subcommand() {
            Some(("list", _)) | None => region::list(new_client()),
            Some(("create", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Region name is mandatory");
                region::create(client, name)
            }
            Some(("delete", args)) => {
                let client = new_client();
                let name = args
                    .get_one::<String>("name")
                    .map(|s| s.as_str())
                    .expect("Region name is mandatory");
                region::delete(client, name)
            }
            _ => unreachable!("subcommand_required prevents `None` or other options"),
        },
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

fn parse_bitrate(val: &str) -> Result<u64, String> {
    let num_end = val.find(|c: char| !c.is_ascii_digit()).unwrap_or(val.len());
    let (num, unit) = val.split_at(num_end);
    if let Ok(num) = num.parse::<u64>() {
        match unit {
            "k" | "kb" | "kbps" => Ok(1000 * num),
            "ki" | "kib" => Ok(1024 * num),
            "M" | "Mb" | "Mbps" => Ok(1000 * 1000 * num),
            "Mi" | "Mib" => Ok(1024 * 1024 * num),
            "" => Ok(num),
            _ => Err(format!("Invalid bitrate: {}", val)),
        }
    } else {
        Err(format!("Invalid bitrate: {}", val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_bitrate() {
        fn test_bitrate(bitrate: &str, res: Result<u64, String>) {
            assert_eq!(
                parse_bitrate(bitrate),
                res,
                "Got wrong result when parsing {}",
                bitrate,
            );
        }
        test_bitrate("1024", Ok(1024));
        test_bitrate("1000", Ok(1000));
        test_bitrate("1k", Ok(1000));
        test_bitrate("1kb", Ok(1000));
        test_bitrate("1kbps", Ok(1000));
        test_bitrate("1Mbps", Ok(1_000_000));
        test_bitrate("1Mb", Ok(1_000_000));
        test_bitrate("1Mib", Ok(1024 * 1024));
        test_bitrate("1Mib", Ok(1024 * 1024));
        test_bitrate("12345Mib", Ok(12345 * 1024 * 1024));
        test_bitrate("1mib", Err("Invalid bitrate: 1mib".to_owned()));
        test_bitrate("", Err("Invalid bitrate: ".to_owned()));
        test_bitrate("1 Kbps", Err("Invalid bitrate: 1 Kbps".to_owned()));
        test_bitrate("1 Kbps", Err("Invalid bitrate: 1 Kbps".to_owned()));
        test_bitrate("1Kbps", Err("Invalid bitrate: 1Kbps".to_owned()));
    }
}
