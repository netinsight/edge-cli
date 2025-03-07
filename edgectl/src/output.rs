use std::collections::BTreeMap;
use std::fmt;
use std::process;

use anyhow::{anyhow, Context};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{
    EdgeClient, Group, Input, Output, OutputAdminStatus, OutputHealthState, OutputPort,
    OutputPortFec, RtpOutputPort, SrtCallerOutputPort, SrtKeylen, SrtListenerOutputPort,
    SrtOutputPort, SrtRateLimiting, UdpOutputPort, ZixiOutputPort,
};

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

pub fn list(client: EdgeClient) {
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

pub fn list_wide(client: EdgeClient) {
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

pub fn show(client: EdgeClient, name: &str) {
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

pub enum NewOutputMode {
    Udp(NewUdpOutputMode),
    Rtp(NewRtpOutputMode),
    Srt(NewSrtOutputMode),
}

pub struct NewUdpOutputMode {
    pub address: String,
    pub port: u16,
    pub source_addr: Option<String>,
}

pub struct NewRtpOutputMode {
    pub address: String,
    pub port: u16,
    pub fec: Option<Fec>,
    pub source_addr: Option<String>,
}

#[derive(Clone)]
pub struct Fec {
    pub mode: FecMode,
    pub rows: u8,
    pub cols: u8,
}

#[derive(Clone)]
pub enum FecMode {
    OneD, // 1D
    TwoD, // 2D
}

pub enum NewSrtOutputMode {
    Listener { port: u16 },
    Caller { address: String, port: u16 },
}

pub struct NewOutput {
    pub name: String,
    pub appliance: String,
    pub interface: String,
    pub input: String,
    pub mode: NewOutputMode,
}

pub fn create(client: EdgeClient, new_output: NewOutput) {
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
                whitelist_cidr_block: vec!["0.0.0.0/0".to_owned()],
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

pub fn delete(client: &EdgeClient, name: &str) -> anyhow::Result<()> {
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
