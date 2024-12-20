// use std::collections::BTreeMap;
use std::fmt;
use std::process;

use tabled::{builder::Builder, settings::Style};

use crate::edge::{
    EdgeClient, Output, OutputAdminStatus, OutputHealthState, OutputPort, OutputPortFec,
    SrtOutputPort, ZixiOutputPort,
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

pub fn list_wide(_client: EdgeClient) {
    todo!()
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
                    println!("  - Mode:             UDP");
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
                    println!("  - Mode:             RTP");
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

pub fn delete(client: EdgeClient, name: &str) -> Result<(), reqwest::Error> {
    let outputs = match client.find_outputs(name) {
        Ok(outputs) => outputs,
        Err(e) => {
            println!("Failed to list outputs for deleteion: {}", e);
            process::exit(1);
        }
    };
    if outputs.is_empty() {
        eprintln!("Output not found: {}", name);
        process::exit(1);
    }
    for output in outputs {
        if let Err(e) = client.delete_output(&output.id) {
            println!("Failed to delete output {}: {}", output.name, e);
            process::exit(1);
        }
        println!("Deleted output {}", output.name);
    }

    Ok(())
}
