use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::BTreeMap;
use std::fmt;
use std::process;

use anyhow::{anyhow, Context};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{
    new_client, AppliancePhysicalPort, DerivableInputSource, EdgeClient, GeneratorBitrate,
    GeneratorBitrateCBR, GeneratorInputPort, IngestTransform, InputAdminStatus, NewInputPort,
    PidMap, RistInputPort, RtpInputPort, SdiEncoderAudioStream, SdiEncoderSettings, SdiInputPort,
    SrtInputPort, UdpInputPort,
};

impl fmt::Display for crate::edge::InputHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.state == "allOk" {
            write!(f, "\x1b[32m✓\x1b[0m")
        } else {
            write!(f, "\x1b[31m✗\x1b[0m {}", self.title)
        }
    }
}

pub(crate) fn subcommand() -> clap::Command {
    Command::new("input")
        .about("Manage inputs")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .arg(
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
                        .required(false)
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
                            "srt",
                            "sdi",
                            "rist",
                            "generator",
                            "derived",
                        ]))
                        .help("The input mode"),
                )
                .arg(
                    Arg::new("interface")
                        .short('i')
                        .long("interface")
                        .required(false)
                        .required_if_eq_any([
                            ("mode", "rtp"),
                            ("mode", "udp"),
                            ("mode", "srt"),
                            ("mode", "rist"),
                            ("mode", "sdi"),
                        ])
                        .help("The interface on the appliance to create the input on"),
                )
                .arg(
                    Arg::new("thumbnail")
                        .long("thumbnail")
                        .value_parser(clap::builder::PossibleValuesParser::new([
                            "core", "edge", "none",
                        ]))
                        .default_value("edge")
                        .help("Set thumbnailing mode"),
                )
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .value_parser(clap::value_parser!(u16).range(1..))
                        .action(clap::ArgAction::Set)
                        .required(false)
                        .required_if_eq_any([
                            ("mode", "rtp"),
                            ("mode", "udp"),
                            ("mode", "rist"),
                            ("listener", "true"),
                        ])
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
                        .value_parser(|val: &str| -> Result<Bitrate, String> {
                            if val == "vbr" {
                                Ok(Bitrate::Vbr)
                            } else {
                                parse_bitrate(val).map(Bitrate::Cbr)
                            }
                        })
                        .help("Set bitrate for generator"),
                )
                .arg(
                    Arg::new("parent")
                        .long("parent")
                        .num_args(1)
                        .help("The parent input for derived inputs. Requires --mode derived"),
                )
                .arg(
                    Arg::new("map")
                        .long("map")
                        .num_args(2)
                        .action(ArgAction::Append)
                        .value_parser(clap::value_parser!(u16).range(1..))
                        .help("Map PIDs in the stream (derived streams only)"),
                )
                .arg(
                    Arg::new("set-null")
                        .long("set-null")
                        .action(ArgAction::Append)
                        .value_parser(clap::value_parser!(u16).range(1..))
                        .help("Replace PID with null packets (derived streams only)"),
                )
                .arg(
                    Arg::new("delete")
                        .long("delete")
                        .action(ArgAction::Append)
                        .value_parser(clap::value_parser!(u16).range(1..))
                        .help("Delete PID from stream (derived streams only)"),
                )
                .arg(
                    Arg::new("caller")
                        .long("caller")
                        .num_args(0)
                        .help("Use an SRT caller. Only applicable for SRT inputs."),
                )
                .arg(
                    Arg::new("listener")
                        .long("listener")
                        .num_args(0)
                        .help("Use an SRT listener. Only applicable for SRT inputs."),
                )
                .arg(
                    Arg::new("rendezvous")
                        .long("rendezvous")
                        .num_args(0)
                        .help("Use an SRT rendezvous. Only applicable for SRT inputs."),
                )
                .arg(Arg::new("destination")
                    .long("dest")
                    .required(false)
                    .required_if_eq("caller", "true")
                    .help("The destination to for SRT callers format ip:port, e.g. 198.51.100.12:4000"),
                )
                .group(
                    clap::ArgGroup::new("srt_mode")
                        .args(["caller", "listener", "rendezvous"])
                        .required(false)
                ),
        )
        .subcommand(
            Command::new("delete").arg(
                Arg::new("name")
                    .required(true)
                    .num_args(1..)
                    .help("The name of the inputs to remove"),
            ),
        )
}

pub(crate) fn run(subcmd: &ArgMatches) {
    match subcmd.subcommand() {
        Some(("list", args)) => {
            let client = new_client();
            if let Err(e) = match args.get_one::<String>("output").map(|s| s.as_str()) {
                Some("wide") => list_wide(client),
                _ => list(client),
            } {
                eprintln!("Failed to list inputs: {:?}", e);
                process::exit(1);
            }
        }
        Some(("show", args)) => {
            let client = new_client();
            let name = args
                .get_one::<String>("name")
                .map(|s| s.as_str())
                .expect("input name should not be None");

            show(client, name);
        }
        Some(("create", args)) => {
            let client = new_client();
            let name = args
                .get_one::<String>("name")
                .map(|s| s.as_str())
                .expect("name is required");
            let port = args.get_one::<u16>("port");
            let mode = args
                .get_one::<String>("mode")
                .map(|s| s.as_str())
                .expect("mode is required");
            let multicast = args.get_one::<String>("multicast").map(|s| s.as_str());
            let bitrate = args.get_one::<Bitrate>("bitrate");

            if port.is_some() && !matches!(mode, "rtp" | "udp" | "srt" | "rist") {
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

            let mode = match mode {
                "rtp" => {
                    let port = match port {
                        Some(p) => p,
                        None => {
                            eprintln!("Port is required for RTP inputs");
                            process::exit(1);
                        }
                    };
                    NewInputMode::Rtp(NewRtpInputMode {
                        appliance: args
                            .get_one::<String>("appliance")
                            .cloned()
                            .expect("appliance is required"),
                        interface: args
                            .get_one::<String>("interface")
                            .cloned()
                            .expect("interface is required"),
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
                    NewInputMode::Udp(NewUdpInputMode {
                        appliance: args
                            .get_one::<String>("appliance")
                            .cloned()
                            .expect("appliance is required"),
                        interface: args
                            .get_one::<String>("interface")
                            .cloned()
                            .expect("interface is required"),
                        port: *port,
                        multicast_address: multicast.map(|s| s.to_owned()),
                    })
                }
                "srt" => {
                    if args.get_flag("rendezvous") {
                        eprintln!("--rendezvous is not yet implemented");
                        process::exit(1);
                    } else if args.get_flag("caller") {
                        let dest = match args.get_one::<String>("destination") {
                            Some(d) => d,
                            None => {
                                eprintln!("Dest is required for SRT caller inputs");
                                process::exit(1);
                            }
                        };
                        let address = dest.split(':').next().expect("dest address is missing");
                        let port = dest
                            .split(':')
                            .next_back()
                            .expect("Port number is required for --dest")
                            .parse::<u16>()
                            .expect("port needs to be a number between 0 and 65535");

                        NewInputMode::Srt(NewSrtInputMode::Caller {
                            appliance: args
                                .get_one::<String>("appliance")
                                .cloned()
                                .expect("appliance is required"),
                            interface: args
                                .get_one::<String>("interface")
                                .cloned()
                                .expect("interface is required"),
                            address: address.to_owned(),
                            port,
                        })
                    } else if args.get_flag("listener") {
                        let port = match args.get_one::<u16>("port") {
                            Some(port) => port,
                            None => {
                                eprintln!("--port is required for srt listener outputs");
                                process::exit(1);
                            }
                        };
                        NewInputMode::Srt(NewSrtInputMode::Listener {
                            appliance: args
                                .get_one::<String>("appliance")
                                .cloned()
                                .expect("appliance is required"),
                            interface: args
                                .get_one::<String>("interface")
                                .cloned()
                                .expect("interface is required"),
                            port: *port,
                        })
                    } else {
                        eprintln!("Missing either --listener, --caller or --rendezvous flag for creating SRT input");
                        process::exit(1);
                    }
                }
                "rist" => {
                    let port = match args.get_one::<u16>("port") {
                        Some(port) => port,
                        None => {
                            eprintln!("--port is required for RIST outputs");
                            process::exit(1);
                        }
                    };
                    NewInputMode::Rist(NewRistInputMode {
                        appliance: args
                            .get_one::<String>("appliance")
                            .cloned()
                            .expect("appliance is required"),
                        interface: args
                            .get_one::<String>("interface")
                            .cloned()
                            .expect("interface is required"),
                        port: *port,
                    })
                }
                "sdi" => NewInputMode::Sdi(NewSdiInputMode {
                    appliance: args
                        .get_one::<String>("appliance")
                        .cloned()
                        .expect("appliance is required"),
                    interface: args
                        .get_one::<String>("interface")
                        .cloned()
                        .expect("interface is required"),
                }),
                "generator" => {
                    if args.contains_id("interface") {
                        eprintln!("Cannot specify interface for generator input");
                        process::exit(1)
                    }

                    NewInputMode::Generator(NewGeneratorInputMode {
                        appliance: args
                            .get_one::<String>("appliance")
                            .cloned()
                            .expect("appliance is required"),
                        bitrate: bitrate.unwrap_or(&Bitrate::Vbr).clone(),
                    })
                }
                "derived" => {
                    let mut rules: Vec<PIDRule> = Vec::new();
                    let maps = args
                        .get_occurrences::<u16>("map")
                        .unwrap_or_default()
                        .map(Iterator::collect)
                        .map(|m: Vec<&u16>| PIDRule::Map(*m[0], *m[1]));
                    let deletes = args
                        .get_many::<u16>("delete")
                        .unwrap_or_default()
                        .map(|d| PIDRule::Delete(*d));

                    let nulls = args
                        .get_many::<u16>("set-null")
                        .unwrap_or_default()
                        .map(|d| PIDRule::SetNull(*d));

                    rules.extend(maps);
                    rules.extend(deletes);
                    rules.extend(nulls);

                    NewInputMode::Derived(NewDerivedInputMode {
                        parent: args
                            .get_one::<String>("parent")
                            .expect("parent is required for derived inputs")
                            .to_owned(),
                        pid_rules: rules,
                    })
                }
                e => {
                    eprintln!("Invalid mode: {}", e);
                    process::exit(1);
                }
            };

            let thumbnail_mode = match args.get_one::<String>("thumbnail").map(|s| s.as_str()) {
                Some("edge") => ThumbnailMode::Edge,
                Some("core") => ThumbnailMode::Core,
                Some("none") => ThumbnailMode::None,
                _ => ThumbnailMode::Edge,
            };

            create(
                client,
                NewInput {
                    name: name.to_owned(),
                    thumbnails: thumbnail_mode,
                    mode,
                },
            )
        }
        Some(("delete", args)) => {
            let client = new_client();
            let mut failed = false;
            for name in args
                .get_many::<String>("name")
                .expect("Input name is mandatory")
            {
                if let Err(e) = delete(&client, name) {
                    eprintln!("Failed to delete input {}: {}", name, e);
                    failed = true;
                }
            }
            if failed {
                process::exit(1);
            }
        }
        Some((cmd, _)) => {
            eprintln!("Command input {cmd} is not yet implemented");
            process::exit(1);
        }
        None => unreachable!("subcommand_required prevents `None`"),
    }
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

fn list(client: EdgeClient) -> anyhow::Result<()> {
    let inputs = client.list_inputs().context("Failed to list edge inputs")?;
    let mut builder = Builder::default();
    builder.push_record(["ID", "Name", "Health"]);

    for input in inputs {
        builder.push_record([
            input.id,
            input.name,
            if input.health.state == "allOk" {
                "\x1b[32m✓\x1b[0m".to_owned()
            } else {
                format!("\x1b[31m✗\x1b[0m {}", input.health.title)
            },
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);

    Ok(())
}

fn list_wide(client: EdgeClient) -> anyhow::Result<()> {
    let inputs = client.list_inputs().context("Failed to list inputs")?;
    let mut groups = BTreeMap::new();
    let mut group_list = client.list_groups().context("Failed to list groups")?;
    while let Some(group) = group_list.pop() {
        groups.insert(group.id.to_owned(), group);
    }

    let mut builder = Builder::default();
    builder.push_record([
        "ID",
        "Name",
        "Group",
        "Enabled",
        "Buffer",
        "Preview",
        "Thumbnails",
        "TR 101 290",
        "Appliances",
        "Health",
    ]);

    for input in inputs {
        builder.push_record([
            input.id,
            input.name,
            groups
                .get(&input.owner)
                .map(|g| g.name.to_owned())
                .unwrap_or("?".to_owned()),
            input.admin_status.to_string(),
            input.buffer_size.to_string(),
            input
                .preview_settings
                .map(|p| p.mode)
                .unwrap_or("unknown".to_owned()),
            input.thumbnail_mode.to_string(),
            if input.tr101290_enabled {
                "on".to_owned()
            } else {
                "off".to_owned()
            },
            input
                .appliances
                .into_iter()
                .map(|a| a.name)
                .collect::<Vec<String>>()
                .join(", "),
            if input.health.state == "allOk" {
                "\x1b[32m✓\x1b[0m".to_owned()
            } else {
                format!("\x1b[31m✗\x1b[0m {}", input.health.title)
            },
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);

    Ok(())
}

fn show(client: EdgeClient, name: &str) {
    let inputs = client.find_inputs(name);
    let inputs = match inputs {
        Ok(inputs) => inputs,
        Err(e) => {
            println!("Failed to find inputs: {:?}", e);
            process::exit(1);
        }
    };

    for input in inputs {
        let group = client.get_group(&input.owner);
        let group_name = group.map(|g| g.name).unwrap_or("unknown".to_owned());

        println!("ID:             {}", input.id);
        println!("Name:           {}", input.name);
        println!("Admin status:   {}", input.admin_status);
        println!("Owner:          {}", group_name);
        println!("Buffer:         {}", input.buffer_size);
        println!(
            "Preview:        {}",
            input
                .preview_settings
                .map(|p| p.mode)
                .unwrap_or("unknown".to_owned())
        );
        println!("Thumbnail mode: {}", input.thumbnail_mode);
        println!("TR 101 290:     {}", input.tr101290_enabled);
        println!(
            "Appliances:     {}",
            input
                .appliances
                .into_iter()
                .map(|a| a.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
        if let Some(ports) = input.ports {
            println!("Ports:");
            for port in ports {
                let port_details = client.get_port(&port.physical_port);
                let name = port_details.map(|p| p.name).unwrap_or("unknown".to_owned());
                println!("  - Mode:                   {}", port.mode);
                println!("    Source interface:       {}", name);
                println!("    Copies:                 {}", port.copies);
            }
        }
        println!("Created:        {}", input.created_at);
        println!("Updated:        {}", input.updated_at);
        println!("Health:         {}", input.health);
    }
}

struct NewInput {
    pub name: String,
    pub thumbnails: ThumbnailMode,
    pub mode: NewInputMode,
}

enum ThumbnailMode {
    Core,
    Edge,
    None,
}

enum NewInputMode {
    Rtp(NewRtpInputMode),
    Udp(NewUdpInputMode),
    Sdi(NewSdiInputMode),
    Srt(NewSrtInputMode),
    Rist(NewRistInputMode),
    Generator(NewGeneratorInputMode),
    Derived(NewDerivedInputMode),
}

struct NewRtpInputMode {
    pub appliance: String,
    pub interface: String,
    pub port: u16,
    pub fec: bool,
    pub multicast_address: Option<String>,
}

struct NewUdpInputMode {
    pub appliance: String,
    pub interface: String,
    pub port: u16,
    pub multicast_address: Option<String>,
}

struct NewRistInputMode {
    pub appliance: String,
    pub interface: String,
    pub port: u16,
}

enum NewSrtInputMode {
    Caller {
        appliance: String,
        interface: String,
        address: String,
        port: u16,
    },
    Listener {
        appliance: String,
        interface: String,
        port: u16,
    },
}

struct NewSdiInputMode {
    pub appliance: String,
    pub interface: String,
}

struct NewGeneratorInputMode {
    pub appliance: String,
    pub bitrate: Bitrate,
}

struct NewDerivedInputMode {
    pub parent: String,
    pub pid_rules: Vec<PIDRule>,
}

#[derive(Debug)]
enum PIDRule {
    Map(u16, u16),
    Delete(u16),
    SetNull(u16),
}

#[derive(Clone)]
enum Bitrate {
    Vbr,
    Cbr(u64),
}

fn create(client: EdgeClient, new_input: NewInput) {
    let ports = match new_input.mode {
        NewInputMode::Rtp(ref rtp) => {
            let interface = get_physical_port(&client, &rtp.appliance, &rtp.interface);
            vec![NewInputPort::Rtp(RtpInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                address: interface
                    .addresses
                    .first()
                    .expect("Expected at least one address on the appliance physical port")
                    .address
                    .to_owned(),
                port: rtp.port,
                fec: rtp.fec,
                multicast_address: rtp.multicast_address.clone(),
                whitelist_cidr_block: Some(vec!["0.0.0.0/0".to_owned()]),
            })]
        }
        NewInputMode::Udp(ref udp) => {
            let interface = get_physical_port(&client, &udp.appliance, &udp.interface);
            vec![NewInputPort::Udp(UdpInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                address: interface
                    .addresses
                    .first()
                    .expect("Expected at least one address on the appliance physical port")
                    .address
                    .to_owned(),
                port: udp.port,
                multicast_address: udp.multicast_address.clone(),
            })]
        }
        NewInputMode::Srt(NewSrtInputMode::Caller {
            ref appliance,
            ref interface,
            ref address,
            port,
        }) => {
            let interface = get_physical_port(&client, appliance, interface);
            vec![NewInputPort::Srt(SrtInputPort::Caller {
                physical_port: interface.id.to_owned(),
                remote_ip: address.to_owned(),
                remote_port: port,
                latency: 120,
                reduced_bitrate_detection: false,
                unrecovered_packets_detection: false,
            })]
        }
        NewInputMode::Srt(NewSrtInputMode::Listener {
            ref appliance,
            ref interface,
            port,
        }) => {
            let interface = get_physical_port(&client, appliance, interface);
            vec![NewInputPort::Srt(SrtInputPort::Listener {
                physical_port: interface.id.to_owned(),
                local_ip: interface.addresses[0].address.to_owned(),
                local_port: port,

                latency: 120,
                reduced_bitrate_detection: false,
                unrecovered_packets_detection: false,
                whitelist_cidr_block: Some(vec!["0.0.0.0/0".to_owned()]),
            })]
        }
        NewInputMode::Rist(NewRistInputMode {
            ref appliance,
            ref interface,
            port,
        }) => {
            let interface = get_physical_port(&client, appliance, interface);
            vec![NewInputPort::Rist(RistInputPort {
                physical_port: interface.id.to_owned(),
                address: interface
                    .addresses
                    .first()
                    .expect("Expected at least one address on the appliance physical port")
                    .address
                    .to_owned(),
                port,
                profile: "simple".to_owned(),
                whitelist_cidr_block: Some(vec!["0.0.0.0/0".to_owned()]),
            })]
        }
        NewInputMode::Sdi(ref sdi) => {
            let interface = get_physical_port(&client, &sdi.appliance, &sdi.interface);
            vec![NewInputPort::Sdi(SdiInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                encoder_settings: SdiEncoderSettings {
                    video_codec: "h.264".to_owned(),
                    total_bitrate: 15000000,
                    gop_size_frames: 150,
                    audio_streams: vec![SdiEncoderAudioStream {
                        codec: "aes3".to_owned(),
                        pair: 1,
                        bitrate: 1920,
                        kind: "stereo".to_owned(),
                    }],
                },
            })]
        }
        NewInputMode::Generator(ref generator) => {
            let interface = get_physical_port(&client, &generator.appliance, "lo");
            vec![NewInputPort::Generator(GeneratorInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                bitrate: match generator.bitrate {
                    Bitrate::Vbr => GeneratorBitrate::Vbr,
                    Bitrate::Cbr(bitrate) => GeneratorBitrate::Cbr(GeneratorBitrateCBR { bitrate }),
                },
            })]
        }
        NewInputMode::Derived(_) => Vec::new(),
    };

    let derive_from = if let NewInputMode::Derived(derived) = new_input.mode {
        let parent = client
            .find_inputs(&derived.parent)
            .expect("Failed to list inputs")
            .into_iter()
            .find(|i| i.name == derived.parent)
            .expect("Could not find parent input");
        Some(DerivableInputSource {
            parent_input: parent.id,
            delay: 1000,
            ingest_transform: Some(IngestTransform::MptsDemuxTransform {
                services: vec![1], // TODO
                pid_map: Some(PidMap {
                    rules: derived
                        .pid_rules
                        .iter()
                        .map(|r| match r {
                            PIDRule::Map(from, to) => crate::edge::PIDRule::Map {
                                pid: *from,
                                dest_pid: *to,
                            },
                            PIDRule::Delete(pid) => crate::edge::PIDRule::Delete { pid: *pid },
                            PIDRule::SetNull(pid) => crate::edge::PIDRule::SetNull { pid: *pid },
                        })
                        .collect(),
                }),
            }),
        })
    } else {
        None
    };

    if let Err(e) = client.create_input(crate::edge::NewInput {
        name: new_input.name,
        tr101290_enabled: true,
        broadcast_standard: "dvb".to_owned(),
        thumbnail_mode: match new_input.thumbnails {
            ThumbnailMode::Core => crate::edge::ThumbnailMode::Core,
            ThumbnailMode::Edge => crate::edge::ThumbnailMode::Edge,
            ThumbnailMode::None => crate::edge::ThumbnailMode::None,
        },
        video_preview_mode: if let ThumbnailMode::Core = new_input.thumbnails {
            "on demand".to_owned()
        } else {
            "off".to_owned()
        },
        admin_status: InputAdminStatus::On,
        ports,
        buffer_size: 6_000,
        max_bitrate: None,
        derive_from,
    }) {
        eprintln!("Failed to create input: {}", e);
        process::exit(1);
    }
}

fn get_physical_port(
    client: &EdgeClient,
    appliance: &str,
    interface: &str,
) -> AppliancePhysicalPort {
    let appl = match client.find_appliances(appliance) {
        Ok(appls) if appls.is_empty() => {
            println!("Could not find appliance {}", appliance);
            process::exit(1);
        }
        Ok(appls) if appls.len() > 1 => {
            println!(
                "Found more than one appliance matching {}: {}",
                appliance,
                appls
                    .into_iter()
                    .map(|a| a.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            process::exit(1);
        }
        Ok(mut appls) => appls.pop().unwrap(),
        Err(e) => {
            println!("Failed to find appliance {}: {}", appliance, e);
            process::exit(1);
        }
    };
    match appl
        .physical_ports
        .into_iter()
        .find(|p| p.name == interface)
    {
        Some(interface) => interface,
        None => {
            println!(
                "Failed to find interface {} on appliance {}",
                interface, appl.name
            );
            process::exit(1);
        }
    }
}

fn delete(client: &EdgeClient, name: &str) -> anyhow::Result<()> {
    let inputs = client.find_inputs(name).context("Failed to find inputs")?;
    if inputs.is_empty() {
        return Err(anyhow!("Input not found"));
    }
    for input in inputs {
        client
            .delete_input(&input.id)
            .context("Failed to delete input")?;
        println!("Deleted input {}", input.name);
    }

    Ok(())
}
