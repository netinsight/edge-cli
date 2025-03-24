use std::collections::BTreeMap;
use std::fmt;
use std::process;

use anyhow::{anyhow, Context};
use clap::{Arg, ArgMatches, Command};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{
    new_client, EdgeClient, Group, Input, Output, OutputAdminStatus, OutputHealthState, OutputPort,
    OutputPortFec, RistOutputPort, RtpOutputPort, SrtCallerOutputPort, SrtKeylen,
    SrtListenerOutputPort, SrtOutputPort, SrtRateLimiting, UdpOutputPort, ZixiOutputPort,
};

pub(crate) fn subcommand() -> clap::Command {
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
                            "rtp", "udp", "sdi", "srt", "rist",
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
                    Arg::new("port")
                        .long("port")
                        .required(false)
                        .value_parser(clap::value_parser!(u16).range(1..))
                        .help("The port to listen on. Only applicable for SRT listeners"),
                )
                .arg(
                    Arg::new("source")
                        .long("source")
                        .help("The source IP address. Applicable for UDP and RTP."),
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
                ).arg(
                    Arg::new("caller")
                        .long("caller")
                        .num_args(0)
                        .help("Use an SRT caller. Only applicable for SRT outputs."),
                ).arg(
                    Arg::new("listener")
                        .long("listener")
                        .num_args(0)
                        .help("Use an SRT listener. Only applicable for SRT outputs."),
                ).arg(
                    Arg::new("rendezvous")
                        .long("rendezvous")
                        .num_args(0)
                        .help("Use an SRT rendezvous. Only applicable for SRT outputs."),
                ),
        )
        .subcommand(
            Command::new("delete").arg(
                Arg::new("name")
                    .required(true)
                    .num_args(1..)
                    .help("The name of the outputs to remove"),
            ),
        )
}

pub(crate) fn run(subcmd: &ArgMatches) {
    match subcmd.subcommand() {
        Some(("list", args)) => {
            let client = new_client();
            match args.get_one::<String>("output").map(|s| s.as_str()) {
                Some("wide") => list_wide(client),
                _ => list(client),
            };
        }
        Some(("show", args)) => {
            let client = new_client();
            let name = args
                .get_one::<String>("name")
                .map(|s| s.as_str())
                .expect("output name should not be None");

            show(client, name);
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

            let source = args.get_one::<String>("source").cloned();

            if args.get_one::<String>("fec").is_some() && mode != "rtp" {
                eprintln!("The --fec argument is only supported for --mode rtp");
                process::exit(1);
            }

            if source.is_some() {
                match mode {
                    "rtp" | "udp" | "rist" => {}
                    _ => {
                        eprintln!("The --source flag is only supported for RTP and UDP outputs");
                        process::exit(1);
                    }
                }
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
                                (Some(rows), Some(cols)) => Fec {
                                    mode: match fec.as_ref() {
                                        "1D" => FecMode::OneD,
                                        "2D" => FecMode::TwoD,
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

                    NewOutputMode::Rtp(NewRtpOutputMode {
                        address: address.to_owned(),
                        port,
                        fec,
                        source_addr: source,
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
                    NewOutputMode::Udp(NewUdpOutputMode {
                        address: address.to_owned(),
                        port,
                        source_addr: source,
                    })
                }
                "srt" => {
                    if args.get_flag("caller") {
                        let dest = match args.get_one::<String>("destination") {
                            Some(d) => d,
                            None => {
                                eprintln!("Dest is required for SRT caller outputs");
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

                        NewOutputMode::Srt(NewSrtOutputMode::Caller {
                            address: address.to_owned(),
                            port,
                        })
                    } else if args.get_flag("rendezvous") {
                        eprintln!("--rendezvous is not yet implemented");
                        process::exit(1);
                    } else if args.get_flag("listener") {
                        let port = match args.get_one::<u16>("port") {
                            Some(port) => port,
                            None => {
                                eprintln!("--port is required for srt listener outputs");
                                process::exit(1);
                            }
                        };
                        NewOutputMode::Srt(NewSrtOutputMode::Listener { port: *port })
                    } else {
                        eprintln!("Need to specify either --caller, --listener or --rendezvous for SRT output");
                        process::exit(1);
                    }
                }
                "rist" => {
                    let dest = match dest {
                        Some(d) => d,
                        None => {
                            eprintln!("Dest is required for RIST outputs");
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
                    NewOutputMode::Rist(NewRistOutputMode {
                        address: address.to_owned(),
                        port,
                        source_addr: source,
                    })
                }
                e => {
                    eprintln!("Invalid mode: {}", e);
                    process::exit(1);
                }
            };

            create(
                client,
                NewOutput {
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
            let mut failed = false;
            for name in args
                .get_many::<String>("name")
                .expect("Output name is mandatory")
            {
                if let Err(e) = delete(&client, name) {
                    eprintln!("Failed to delete output {}: {}", name, e);
                    failed = true;
                }
            }
            if failed {
                process::exit(1);
            }
        }
        Some((cmd, _)) => {
            eprintln!("Command output {cmd} is not yet implemented");
            process::exit(1);
        }
        None => unreachable!("subcommand_required prevents `None`"),
    }
}

impl fmt::Display for OutputHealthState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotConfigured => write!(f, "\x1b[31m✗\x1b[0m Not configured"),
            Self::MetricsMissing => write!(f, "\x1b[31m✗\x1b[0m Missing metrics"),
            Self::Tr101290Priority1Error => {
                write!(f, "\x1b[31m✗\x1b[0m TR 101 290 Priority 1 errors")
            }
            Self::ReducedRedundancy => write!(f, "\x1b[33m⚠\x1b[0m Reduced redundancy"),
            Self::AllOk => write!(f, "\x1b[32m✓\x1b[0m"),
            Self::NotAcknowledged => write!(f, "No ACKs recieved"),
            Self::InputError => write!(f, "\x1b[31m✗\x1b[0m Input error"),
            Self::OutputError => write!(f, "\x1b[31m✗\x1b[0m Output error"),
            Self::Alarm => write!(f, "\x1b[31m✗\x1b[0m Alarmc"),
        }
    }
}

impl Output {
    fn health_fmt(&self) -> String {
        match self.admin_status {
            OutputAdminStatus::On => self
                .health
                .as_ref()
                .map(|h| {
                    if h.title.is_empty() {
                        h.state.to_string()
                    } else {
                        format!("{} ({})", h.state, h.title)
                    }
                })
                .unwrap_or("unknown".to_owned()),
            OutputAdminStatus::Off => "\x1b[37m⏻\x1b[0m Disabled".to_owned(),
        }
    }
}

fn list(client: EdgeClient) {
    let outputs = client.list_outputs().expect("Failed to list outputs");
    let mut builder = Builder::default();
    builder.push_record(["ID", "Name", "Health"]);

    for output in outputs {
        let health = output.health_fmt();
        builder.push_record([output.id, output.name, health]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

fn list_wide(client: EdgeClient) {
    let outputs = client.list_outputs().expect("Failed to list outputs");
    let mut builder = Builder::default();
    builder.push_record([
        "ID",
        "Name",
        "Group",
        "Enabled",
        "Input",
        "Redudancy",
        "Delay",
        "Delay,mode",
        "Appliances",
        "Health",
    ]);
    let groups: Vec<String> = outputs.iter().map(|o| o.group.clone()).collect();
    // TODO: Improve performance here by doing a bulk fetch of groups
    let groups: BTreeMap<String, Group> = groups
        .into_iter()
        .filter_map(|id| {
            let group = client.get_group(&id);
            group.map(|group| (id, group)).ok()
        })
        .collect();
    let inputs: Vec<String> = outputs
        .iter()
        .filter_map(|o| o.input.as_ref().map(|i| i.to_owned()))
        .collect();
    // TODO: Improve performance here by doing a bulk fetch of inputs
    let inputs: BTreeMap<String, Input> = inputs
        .into_iter()
        .filter_map(|id| {
            let input = client.get_input(&id);
            input.map(|input| (id, input)).ok()
        })
        .collect();
    for output in outputs {
        let health = output.health_fmt();
        let input = match output.input {
            Some(input) => inputs
                .get(&input)
                .map(|i| i.name.to_owned())
                .unwrap_or(input),
            None => "".to_owned(),
        };
        builder.push_record([
            output.id,
            output.name,
            groups
                .get(&output.group)
                .map(|g| g.name.to_owned())
                .unwrap_or(output.group),
            output.admin_status.to_string(),
            input,
            output
                .redundancy_mode
                .map(|m| m.to_string())
                .unwrap_or("".to_owned()),
            output
                .delay
                .map(|d| format!("{}ms", d))
                .unwrap_or("".to_owned()),
            output
                .delay_mode
                .map(|m| m.to_string())
                .unwrap_or("".to_owned()),
            output
                .appliances
                .into_iter()
                .map(|a| a.name)
                .collect::<Vec<String>>()
                .join(", "),
            health,
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

fn show(client: EdgeClient, name: &str) {
    let outputs = client.find_outputs(name);
    let outputs = match outputs {
        Ok(outputs) => outputs,
        Err(e) => {
            eprintln!("Failed to find output: {:?}", e);
            process::exit(1);
        }
    };

    let many_outputs = outputs.len() > 1;
    for output in outputs {
        let health = output.health_fmt();
        let group = client.get_group(&output.group);
        let group_name = group.map(|g| g.name).unwrap_or("unknown".to_owned());
        let input = output.input.and_then(|input| client.get_input(&input).ok());
        let input = input.map(|input| input.name).unwrap_or("".to_owned());
        let redundancy_mode = output
            .redundancy_mode
            .map(|r| r.to_string())
            .unwrap_or("".to_owned());
        let delay = output
            .delay
            .map(|d| format!("{}ms", d))
            .unwrap_or("".to_owned());
        let delay_mode = output
            .delay_mode
            .map(|m| m.to_string())
            .unwrap_or("".to_owned());
        let misconfigured = output
            .misconfigured
            .map(|m| m.to_string())
            .unwrap_or("unknown".to_owned());
        let alarms = output
            .alarms
            .map(|alarms| {
                alarms
                    .iter()
                    .map(|alarm| {
                        if let Some(text) = &alarm.text {
                            format!("[{}] {}: {}", alarm.alarm_severity, alarm.alarm_cause, text)
                        } else {
                            format!("[{}] {}", alarm.alarm_severity, alarm.alarm_cause)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            })
            .unwrap_or("".to_owned());
        let alarms = if alarms.is_empty() {
            "None".to_owned()
        } else {
            alarms
        };
        let appliances = output
            .appliances
            .iter()
            .map(|appl| appl.name.clone())
            .collect::<Vec<String>>()
            .join(", ");

        println!("ID:             {}", output.id);
        println!("Name:           {}", output.name);
        println!("Input:          {}", input);
        println!("Admin status:   {}", output.admin_status);
        println!("Redudancy:      {}", redundancy_mode);
        println!("Group:          {}", group_name);
        println!("Delay:          {}", delay);
        println!("Delay mode:     {}", delay_mode);

        println!("Ports:");
        for port in output.ports {
            match port {
                OutputPort::Srt(SrtOutputPort::Listener(port)) => {
                    let addr = format!("{}:{}", port.local_ip, port.local_port);
                    println!("  - Mode:             srt");
                    println!("    SRT mode:         listener");
                    println!("    Listening at:     {}", addr);
                }
                OutputPort::Srt(SrtOutputPort::Caller(port)) => {
                    let addr = format!("{}:{}", port.remote_ip, port.remote_port);
                    println!("  - Mode:             srt");
                    println!("    SRT mode:         caller");
                    println!("    Calling:          {}", addr);
                }
                OutputPort::Srt(SrtOutputPort::Rendezvous(port)) => {
                    let source = format!("{}:{}", port.local_ip, port.remote_port);
                    let dest = format!("{}:{}", port.remote_ip, port.remote_port);
                    println!("  - Mode:             srt");
                    println!("    SRT mode:         rendezvous");
                    println!("    Source:           {}", source);
                    println!("    Destination:      {}", dest);
                }
                OutputPort::Zixi(ZixiOutputPort::Pull(port)) => {
                    println!("  - Mode:             Zixi");
                    println!("    Zixi mode:        pull");
                    println!("    Stream ID:        {}", port.stream_id);
                }
                OutputPort::Zixi(ZixiOutputPort::Push(port)) => {
                    let remote = port
                        .link_set_1
                        .iter()
                        .chain(port.link_set_2.unwrap_or(Vec::new()).iter())
                        .map(|ls| format!("{}:{}", ls.remote_ip, ls.remote_port))
                        .collect::<Vec<String>>()
                        .join(", ");
                    println!("  - Mode:             Zixi");
                    println!("    Zixi mode:        push");
                    println!("    Stream ID:        {}", port.stream_id);
                    println!("    Remote addr:      {}", remote);
                }
                OutputPort::Udp(port) => {
                    let addr = format!("{}:{}", port.address, port.port);
                    let iface = client
                        .get_port(&port.physical_port)
                        .map(|iface| iface.name)
                        .unwrap_or(port.physical_port); // fall back to ID if on failure
                    println!("  - Mode:             UDP");
                    println!("    Interface:        {}", iface);
                    println!("    Dest:             {}", addr);
                }
                OutputPort::Rtp(port) => {
                    let addr = format!("{}:{}", port.address, port.port);
                    let fec = match port.fec {
                        Some(fec) => format!(
                            "{}: {}×{}",
                            match fec {
                                OutputPortFec::Fec1D => "1D",
                                OutputPortFec::Fec2D => "2D",
                            },
                            port.fec_rows
                                .map(|r| r.to_string())
                                .unwrap_or("".to_owned()),
                            port.fec_cols
                                .map(|r| r.to_string())
                                .unwrap_or("".to_owned()),
                        ),
                        None => "no".to_owned(),
                    };
                    let iface = client
                        .get_port(&port.physical_port)
                        .map(|iface| iface.name)
                        .unwrap_or(port.physical_port); // fall back to ID if on failure
                    println!("  - Mode:             RTP");
                    println!("    Interface:        {}", iface);
                    println!("    Dest:             {}", addr);
                    println!("    FEC:              {}", fec);
                }
                OutputPort::Sdi(port) => {
                    let physical_port = match client.get_port(&port.physical_port) {
                        Ok(port) => port,
                        Err(e) => {
                            eprintln!("Failed to get physical port {}: {}", port.physical_port, e);
                            process::exit(1);
                        }
                    };
                    println!("  - Mode:             SDI");
                    println!("    Interface:        {}", physical_port.name);
                }
                OutputPort::Rist(port) => {
                    let addr = format!("{}:{}", port.address, port.port);
                    println!("  - Mode:             RIST");
                    println!("    Profile:          {}", port.profile);
                    println!("    Dest:             {}", addr);
                }
                OutputPort::Rtmp(port) => {
                    println!("  - Mode:             RTMP");
                    println!("    Address:          {}", port.rtmp_destination_address);
                }
                _ => println!("- Mode:              Unsupported mode!"), // TODO
            }
        }

        println!("Alarms:         {}", alarms);
        println!("Appliances:     {}", appliances);
        println!("Misconfigured:  {}", misconfigured);
        println!("Created:        {}", output.created_at);
        println!("Updated:        {}", output.updated_at);
        println!("Health:         {}", health);

        if many_outputs {
            println!();
        }
    }
}

enum NewOutputMode {
    Udp(NewUdpOutputMode),
    Rtp(NewRtpOutputMode),
    Srt(NewSrtOutputMode),
    Rist(NewRistOutputMode),
}

struct NewUdpOutputMode {
    pub address: String,
    pub port: u16,
    pub source_addr: Option<String>,
}

struct NewRtpOutputMode {
    pub address: String,
    pub port: u16,
    pub fec: Option<Fec>,
    pub source_addr: Option<String>,
}

#[derive(Clone)]
struct Fec {
    pub mode: FecMode,
    pub rows: u8,
    pub cols: u8,
}

#[derive(Clone)]
enum FecMode {
    OneD, // 1D
    TwoD, // 2D
}

enum NewSrtOutputMode {
    Listener { port: u16 },
    Caller { address: String, port: u16 },
}

struct NewRistOutputMode {
    pub address: String,
    pub port: u16,
    pub source_addr: Option<String>,
}

struct NewOutput {
    pub name: String,
    pub appliance: String,
    pub interface: String,
    pub input: String,
    pub mode: NewOutputMode,
}

fn create(client: EdgeClient, new_output: NewOutput) {
    let appl = match client.find_appliances(&new_output.appliance) {
        Ok(appls) if appls.is_empty() => {
            println!("Could not find appliance {}", new_output.appliance);
            process::exit(1);
        }
        Ok(appls) if appls.len() > 1 => {
            println!(
                "Found more than one appliance matching {}: {}",
                new_output.appliance,
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
            println!("Failed to find appliance {}: {}", new_output.appliance, e);
            process::exit(1);
        }
    };

    let interface = match appl
        .physical_ports
        .iter()
        .find(|p| p.name == new_output.interface)
    {
        Some(interface) => interface,
        None => {
            println!(
                "Failed to find interface {} on appliance {}",
                new_output.interface, appl.name
            );
            process::exit(1);
        }
    };

    let input = match client.find_inputs(&new_output.input) {
        Ok(inputs) if inputs.is_empty() => {
            println!("Could not find input {}", new_output.input);
            process::exit(1);
        }
        Ok(inputs) if inputs.len() > 1 => {
            println!(
                "Found more than one input matching {}: {}",
                new_output.input,
                inputs
                    .into_iter()
                    .map(|a| a.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            process::exit(1);
        }
        Ok(mut inputs) => inputs.pop().unwrap(),
        Err(e) => {
            println!("Failed to find inputs {}: {}", new_output.input, e);
            process::exit(1);
        }
    };

    let ports = match new_output.mode {
        NewOutputMode::Udp(udp) => vec![OutputPort::Udp(UdpOutputPort {
            address: udp.address,
            port: udp.port,
            physical_port: interface.id.to_owned(),
            source_address: udp.source_addr,
        })],
        NewOutputMode::Rtp(rtp) => {
            let fec = rtp.fec.as_ref().map(|fec| match fec.mode {
                FecMode::OneD => OutputPortFec::Fec1D,
                FecMode::TwoD => OutputPortFec::Fec2D,
            });

            vec![OutputPort::Rtp(RtpOutputPort {
                address: rtp.address,
                port: rtp.port,
                physical_port: interface.id.to_owned(),
                fec,
                fec_rows: rtp.fec.as_ref().map(|fec| fec.rows),
                fec_cols: rtp.fec.as_ref().map(|fec| fec.cols),
                source_address: rtp.source_addr,
            })]
        }
        NewOutputMode::Srt(NewSrtOutputMode::Listener { port }) => vec![OutputPort::Srt(
            SrtOutputPort::Listener(SrtListenerOutputPort {
                local_ip: interface.addresses[0].address.to_owned(),
                local_port: port,
                physical_port: interface.id.to_owned(),
                latency: 120,
                pbkeylen: SrtKeylen::None,
                rate_limiting: SrtRateLimiting::NotEnforced,
                whitelist_cidr_block: Some(vec!["0.0.0.0/0".to_owned()]),
            }),
        )],
        NewOutputMode::Srt(NewSrtOutputMode::Caller { address, port }) => vec![OutputPort::Srt(
            SrtOutputPort::Caller(SrtCallerOutputPort {
                physical_port: interface.id.to_owned(),
                remote_ip: address,
                remote_port: port,

                latency: 120,
                pbkeylen: SrtKeylen::None,
                rate_limiting: SrtRateLimiting::NotEnforced,
            }),
        )],
        NewOutputMode::Rist(rist) => vec![OutputPort::Rist(RistOutputPort {
            address: rist.address,
            port: rist.port,
            physical_port: interface.id.to_owned(),
            source_address: rist.source_addr,
            profile: "simple".to_owned(),
        })],
    };

    if let Err(e) = client.create_output(crate::edge::NewOutput {
        name: new_output.name,
        admin_status: OutputAdminStatus::On,
        delay: None,
        delay_mode: None,
        group: None,
        input: input.id,
        redundancy_mode: None,
        tags: Vec::new(),
        ports,
    }) {
        eprintln!("Failed to create output: {}", e);
        process::exit(1);
    }
}

fn delete(client: &EdgeClient, name: &str) -> anyhow::Result<()> {
    let outputs = client
        .find_outputs(name)
        .context("Failed to list outputs")?;
    if outputs.is_empty() {
        return Err(anyhow!("Output not found"));
    }
    for output in outputs {
        client
            .delete_output(&output.id)
            .context("Failed to delete output")?;
        println!("Deleted output {}", output.name);
    }

    Ok(())
}
