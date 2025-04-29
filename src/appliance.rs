use std::{fmt, process};

use anyhow::{anyhow, Context};
use clap::{Arg, ArgMatches, Command};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{new_client, Appliance, ApplianceHealthState, AppliancePortType, EdgeClient};

pub(crate) fn subcommand() -> clap::Command {
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
        .subcommand(
            Command::new("inputs").about("List appliance inputs").arg(
                Arg::new("name")
                    .required(true)
                    .help("The appliance name to show details for"),
            ),
        )
        .subcommand(
            Command::new("outputs").about("List appliance outputs").arg(
                Arg::new("name")
                    .required(true)
                    .help("The appliance name to show details for"),
            ),
        )
        .subcommand(
            Command::new("delete").arg(
                Arg::new("name")
                    .required(true)
                    .num_args(1..)
                    .help("The name of the appliances to delete"),
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
}

pub(crate) fn run(subcmd: &ArgMatches) {
    match subcmd.subcommand() {
        Some(("list", _)) | None => {
            let client = new_client();
            list(client)
        }
        Some(("show", args)) => {
            let client = new_client();
            let name = args
                .get_one::<String>("name")
                .map(|s| s.as_str())
                .expect("Appliance name is mandatory");
            show(client, name)
        }
        Some(("delete", args)) => {
            let client = new_client();
            let mut failed = false;
            for name in args
                .get_many::<String>("name")
                .expect("Appliance name is mandatory")
            {
                if let Err(e) = delete(&client, name) {
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
            inputs(client, name)
        }
        Some(("outputs", args)) => {
            let client = new_client();
            let name = args
                .get_one::<String>("name")
                .map(|s| s.as_str())
                .expect("Appliance name is mandatory");
            outputs(client, name)
        }
        Some(("config", args)) => {
            let client = new_client();
            let name = args
                .get_one::<String>("name")
                .map(|s| s.as_str())
                .expect("Appliance name is mandatory");
            config(client, name)
        }
        Some(("restart", args)) => {
            let client = new_client();
            let name = args
                .get_one::<String>("name")
                .map(|s| s.as_str())
                .expect("Appliance name is mandatory");
            restart(client, name)
        }
        _ => unreachable!("subcommand_required prevents `None` or other options"),
    }
}

impl fmt::Display for ApplianceHealthState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Connected => write!(f, "\x1b[32m✓\x1b[0m connected"),
            Self::Missing => write!(f, "\x1b[31m✗\x1b[0m missing"),
            Self::NeverConnected => write!(f, "\x1b[31m✗\x1b[0m never connected"),
        }
    }
}

impl fmt::Display for AppliancePortType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ip => f.pad("IP"),
            Self::Coax => f.pad("Coax"),
            Self::Videon => f.pad("Videon"),
            Self::Ndi => f.pad("Ndi"),
        }
    }
}

fn list(client: EdgeClient) {
    let appliances = client
        .list_appliances()
        .expect("Failed to fetch appliance list");

    let mut builder = Builder::default();
    builder.push_record(["Name", "ID", "Type", "State"]);
    for appliance in appliances {
        builder.push_record([
            appliance.name,
            appliance.id,
            appliance.kind,
            appliance
                .health
                .map(|h| h.state.to_string())
                .unwrap_or("unknown".to_owned()),
        ])
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

fn show(client: EdgeClient, name: &str) {
    let appliances = match client.find_appliances(name) {
        Ok(appls) => appls,
        Err(e) => {
            println!("Failed to list appliances: {}", e);
            process::exit(1)
        }
    };

    if appliances.is_empty() {
        println!("No appliance found: {}", name);
        process::exit(1)
    }

    for appliance in appliances {
        let group = client.get_group(&appliance.owner);
        let group_name = group.map(|g| g.name).unwrap_or("unknown".to_owned());
        let last_registered_at = appliance.last_registered_at.unwrap_or("unknown".to_owned());
        let health_status = appliance
            .health
            .map(|h| match h.state {
                ApplianceHealthState::Connected => format!("\x1b[32m✓\x1b[0m {}", h.title),
                _ => format!("\x1b[31m✗\x1b[0m {}", h.title),
            })
            .unwrap_or("unknown".to_owned());

        println!("ID:                   {}", appliance.id);
        println!("Name:                 {}", appliance.name);
        println!("Hostname:             {}", appliance.hostname);
        println!("Contact:              {}", appliance.contact);
        println!("Product name;         {}", appliance.kind); // TODO: Pretty-print
        println!("Serial number:        {}", appliance.serial);
        println!("Group:                {}", group_name);
        println!(
            "Version (control):    image={}, software={}",
            appliance
                .version
                .control_image_version
                .unwrap_or("unknown".to_owned()),
            appliance.version.control_software_version
        );
        println!(
            "Version (data):       image={}, software={}",
            appliance
                .version
                .data_image_version
                .unwrap_or("unknown".to_owned()),
            appliance
                .version
                .data_software_version
                .unwrap_or("unknown".to_owned())
        );
        println!("Interfaces:");
        for iface in appliance.physical_ports {
            let networks = iface
                .networks
                .into_iter()
                .map(|n| n.name)
                .collect::<Vec<_>>()
                .join(", ");

            println!("  - Name: {}", iface.name);
            println!("    Type: {}", iface.port_type);
            println!("    Networks: {}", networks);
            println!("    Addresses:");
            for addr in iface.addresses {
                println!("      - Address: {}", addr.address);
                if let Some(public) = addr.public_address {
                    println!("        Public: {}", public);
                }
            }
        }
        println!("Status:               {}", health_status);
        println!("Running since:        {}", last_registered_at);
        if !appliance.alarms.is_empty() {
            println!("Alarms:");
            for alarm in appliance.alarms {
                println!(
                    "  - [{}] {} {}",
                    alarm.time,
                    alarm.alarm_severity.to_uppercase(),
                    alarm.alarm_cause
                );
            }
        }
    }
}

fn delete(client: &EdgeClient, name: &str) -> anyhow::Result<()> {
    let appliances = client
        .find_appliances(name)
        .context("Failed to list appliances for deletion")?;

    if appliances.is_empty() {
        return Err(anyhow!("Appliance not found"));
    }

    for appliance in appliances {
        client
            .delete_appliance(&appliance.id)
            .context("Failed to delete appliance")?;
        println!("Deleted appliance {}", appliance.name);
    }

    Ok(())
}

fn inputs(client: EdgeClient, name: &str) {
    let appliance = get_appliance(&client, name);
    let inputs = match client.get_appliance_inputs(&appliance.id) {
        Ok(inputs) => inputs,
        Err(e) => {
            eprintln!("Failed to list inputs for appliance: {}", e);
            process::exit(1);
        }
    };

    let mut builder = Builder::default();
    builder.push_record(["Name", "Group", "Status"]);

    for input in inputs {
        let group = client.get_group(&input.input_group);
        let group = group.map(|g| g.name).unwrap_or("unknown".to_owned());
        builder.push_record([
            input.input_name,
            group,
            input.input_admin_status.to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

fn outputs(client: EdgeClient, name: &str) {
    let appliance = get_appliance(&client, name);
    let outputs = match client.get_appliance_outputs(&appliance.id) {
        Ok(outputs) => outputs,
        Err(e) => {
            eprintln!("Failed to list inputs for appliance: {}", e);
            process::exit(1);
        }
    };

    let mut builder = Builder::default();
    builder.push_record(["Name", "Group", "Status"]);

    for output in outputs {
        let group = client.get_group(&output.output_group);
        let group = group.map(|g| g.name).unwrap_or("unknown".to_owned());
        builder.push_record([
            output.output_name,
            group,
            output.output_admin_status.to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

fn config(client: EdgeClient, name: &str) {
    let appliance = get_appliance(&client, name);
    let config = match client.get_appliance_config(&appliance.id) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to get appliance config: {}", e);
            process::exit(1);
        }
    };
    println!("{}", serde_json::to_string_pretty(&config).unwrap());
}

fn restart(client: EdgeClient, name: &str) {
    let appliance = get_appliance(&client, name);
    eprintln!("Restarting appliance {}", appliance.name);
    if let Err(e) = client.restart_appliance(&appliance.id) {
        eprintln!("Failed to restart appliance: {}", e);
        process::exit(1);
    }
    eprintln!("Appliance {} restarted", appliance.name)
}

fn get_appliance(client: &EdgeClient, name: &str) -> Appliance {
    let mut appliances = match client.find_appliances(name) {
        Ok(appls) => appls,
        Err(e) => {
            println!("Failed to list appliances for deleteion: {}", e);
            process::exit(1)
        }
    };
    if appliances.len() > 1 {
        eprintln!("Found multiple appliances matching {}:", name);
        for appl in appliances {
            eprintln!("{}", appl.name);
        }
        process::exit(1);
    }
    match appliances.pop() {
        Some(appliance) => appliance,
        None => {
            eprintln!("Appliance not found: {}", name);
            process::exit(1);
        }
    }
}
