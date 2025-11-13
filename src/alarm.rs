use chrono::{DateTime, Utc};
use clap::{Arg, ArgMatches, Command};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use tabled::{builder::Builder, settings::Style};

use crate::edge::new_client;

pub(crate) fn subcommand() -> clap::Command {
    Command::new("alarm")
        .about("Manage alarms")
        .subcommand_required(true)
        .subcommand(
            Command::new("list").about("List active alarms").arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .value_parser(["short", "wide"])
                    .default_value("short")
                    .help("Change the output format"),
            ),
        )
}

pub(crate) fn run(args: &ArgMatches) {
    match args.subcommand() {
        Some(("list", sub_args)) => {
            match sub_args.get_one::<String>("output").map(|s| s.as_str()) {
                Some("wide") => list_wide(),
                _ => list(),
            }
        }
        _ => {
            eprintln!("No subcommand provided");
            std::process::exit(1);
        }
    }
}

fn format_time_ago(time_str: &str) -> String {
    let parsed = time_str.parse::<DateTime<Utc>>();

    match parsed {
        Ok(alarm_time) => {
            let now: DateTime<Utc> = SystemTime::now().into();
            let duration = now.signed_duration_since(alarm_time);

            let seconds = duration.num_seconds();
            let minutes = duration.num_minutes();
            let hours = duration.num_hours();
            let days = duration.num_days();

            if seconds < 60 {
                format!("{}s ago", seconds)
            } else if minutes < 60 {
                format!("{}m ago", minutes)
            } else if hours < 24 {
                format!("{}h ago", hours)
            } else {
                format!("{}d ago", days)
            }
        }
        Err(_) => time_str.to_string(),
    }
}

fn list() {
    let client = new_client();
    let alarms = client.list_alarms().expect("Failed to list alarms");

    if alarms.is_empty() {
        println!("No active alarms found");
        return;
    }

    let inputs = client
        .list_inputs_by_ids(
            alarms
                .iter()
                .filter_map(|a| a.input_id.as_ref())
                .chain(alarms.iter().filter_map(|a| a.affected_input.as_ref()))
                .cloned()
                .collect(),
        )
        .expect("Failed to list inputs");
    let outputs = client
        .list_outputs_by_ids(
            alarms
                .iter()
                .filter_map(|a| a.output_id.as_ref())
                .chain(alarms.iter().filter_map(|a| a.affected_output.as_ref()))
                .cloned()
                .collect(),
        )
        .expect("Failed to list outputs");

    let input_map: HashMap<String, String> = inputs
        .into_iter()
        .map(|input| (input.id, input.name))
        .collect();
    let output_map: HashMap<String, String> = outputs
        .into_iter()
        .map(|output| (output.id, output.name))
        .collect();

    let mut builder = Builder::default();
    builder.push_record([
        "Time Ago",
        "Severity",
        "Cause",
        "Message",
        "Appliance",
        "Entity",
    ]);

    for alarm in alarms {
        let time_ago = alarm
            .raised_at
            .as_ref()
            .map(|t| format_time_ago(t))
            .unwrap_or_else(|| "-".to_string());
        let appliance = alarm.appliance_name.as_deref().unwrap_or("-");

        let entities: HashSet<String> = [
            alarm
                .input_id
                .as_ref()
                .and_then(|id| input_map.get(id))
                .map(|s| format!("input: {}", s)),
            alarm
                .output_id
                .as_ref()
                .and_then(|id| output_map.get(id))
                .map(|s| format!("output: {}", s)),
            alarm
                .affected_input
                .as_ref()
                .and_then(|id| input_map.get(id))
                .map(|s| format!("input: {}", s)),
            alarm
                .affected_output
                .as_ref()
                .and_then(|id| output_map.get(id))
                .map(|s| format!("output: {}", s)),
            alarm.input_name.as_deref().map(|s| format!("input: {}", s)),
            alarm
                .output_name
                .as_deref()
                .map(|s| format!("output: {}", s)),
            alarm
                .affected_input
                .as_ref()
                .map(|s| format!("input: {}", s)),
            alarm
                .affected_output
                .as_ref()
                .map(|s| format!("output: {}", s)),
        ]
        .into_iter()
        .flatten()
        .collect();

        let mut entity_vec: Vec<_> = entities.into_iter().collect();
        entity_vec.sort();
        let entity = if entity_vec.is_empty() {
            "-".to_string()
        } else {
            entity_vec.join(", ")
        };

        let message = alarm.text.as_deref().unwrap_or("-");

        builder.push_record([
            &time_ago,
            &alarm.alarm_severity,
            &alarm.alarm_cause,
            message,
            appliance,
            &entity,
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}

fn list_wide() {
    let client = new_client();
    let alarms = client.list_alarms().expect("Failed to list alarms");

    if alarms.is_empty() {
        println!("No active alarms found");
        return;
    }

    let inputs = client
        .list_inputs_by_ids(
            alarms
                .iter()
                .filter_map(|a| a.input_id.as_ref())
                .chain(alarms.iter().filter_map(|a| a.affected_input.as_ref()))
                .cloned()
                .collect(),
        )
        .expect("Failed to list inputs");
    let outputs = client
        .list_outputs_by_ids(
            alarms
                .iter()
                .filter_map(|a| a.output_id.as_ref())
                .chain(alarms.iter().filter_map(|a| a.affected_output.as_ref()))
                .cloned()
                .collect(),
        )
        .expect("Failed to list outputs");

    let input_map: HashMap<String, String> = inputs
        .into_iter()
        .map(|input| (input.id, input.name))
        .collect();
    let output_map: HashMap<String, String> = outputs
        .into_iter()
        .map(|output| (output.id, output.name))
        .collect();

    let mut builder = Builder::default();
    builder.push_record([
        "Time Ago",
        "Severity",
        "Cause",
        "Message",
        "Appliance",
        "Entity",
        "Type",
        "Object Name",
        "Object Purpose",
        "Port",
        "Repeat",
        "Region",
        "Raised At",
    ]);

    for alarm in alarms {
        let time_ago = alarm
            .raised_at
            .as_ref()
            .map(|t| format_time_ago(t))
            .unwrap_or_else(|| "-".to_string());

        let entities: HashSet<String> = [
            alarm
                .input_id
                .as_ref()
                .and_then(|id| input_map.get(id))
                .map(|s| format!("input: {}", s)),
            alarm
                .output_id
                .as_ref()
                .and_then(|id| output_map.get(id))
                .map(|s| format!("output: {}", s)),
            alarm
                .affected_input
                .as_ref()
                .and_then(|id| input_map.get(id))
                .map(|s| format!("input: {}", s)),
            alarm
                .affected_output
                .as_ref()
                .and_then(|id| output_map.get(id))
                .map(|s| format!("output: {}", s)),
            alarm.input_name.as_deref().map(|s| format!("input: {}", s)),
            alarm
                .output_name
                .as_deref()
                .map(|s| format!("output: {}", s)),
            alarm
                .affected_input
                .as_ref()
                .filter(|s| !s.contains('-'))
                .map(|s| format!("input: {}", s)),
            alarm
                .affected_output
                .as_ref()
                .filter(|s| !s.contains('-'))
                .map(|s| format!("output: {}", s)),
        ]
        .into_iter()
        .flatten()
        .collect();

        let mut entity_vec: Vec<_> = entities.into_iter().collect();
        entity_vec.sort();
        let entity = if entity_vec.is_empty() {
            "-".to_string()
        } else {
            entity_vec.join(", ")
        };

        let port = alarm
            .physical_port_id
            .as_ref()
            .and_then(|port_id| client.get_port(port_id).map(|p| p.name).ok())
            .unwrap_or_else(|| "-".to_string());

        builder.push_record([
            &time_ago,
            &alarm.alarm_severity,
            &alarm.alarm_cause,
            alarm.text.as_deref().unwrap_or("-"),
            alarm.appliance_name.as_deref().unwrap_or("-"),
            &entity,
            &alarm.alarm_type,
            &alarm.object_name,
            alarm.object_purpose.as_deref().unwrap_or("-"),
            &port,
            &alarm.repeat_count.to_string(),
            alarm.region.as_deref().unwrap_or("-"),
            alarm.raised_at.as_deref().unwrap_or("-"),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}
