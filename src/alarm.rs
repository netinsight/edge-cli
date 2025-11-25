use chrono::{DateTime, Utc};
use clap::{Arg, ArgMatches, Command};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use tabled::{builder::Builder, settings::Style};

use crate::edge::new_client;

fn parse_time_filter(input: &str) -> Result<String, String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.to_utc().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());
    }

    let duration = humantime::parse_duration(input).map_err(|e| {
        format!(
            "Invalid time format '{}': {}. Expected ISO8601 timestamp or relative time (e.g., 2h, 30m)",
            input, e
        )
    })?;

    let now = SystemTime::now();
    let past_time = now
        .checked_sub(duration)
        .ok_or_else(|| format!("Time '{}' is too far in the past", input))?;

    let datetime: DateTime<Utc> = past_time.into();
    Ok(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

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
        .subcommand(
            Command::new("history")
                .about("View alarm history")
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_parser(["short", "wide", "detailed"])
                        .default_value("short")
                        .help("Change the output format"),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .short('l')
                        .value_parser(clap::value_parser!(usize))
                        .default_value("30")
                        .help("Maximum number of alarm history entries to fetch"),
                )
                .arg(
                    Arg::new("since")
                        .long("since")
                        .value_name("TIME")
                        .help("Show alarms since this time (e.g., 2h, 30m, 2025-11-14T09:00:00Z)"),
                )
                .arg(
                    Arg::new("until")
                        .long("until")
                        .value_name("TIME")
                        .help("Show alarms until this time (e.g., 2h, 30m, 2025-11-14T09:00:00Z)"),
                )
                .subcommand(
                    Command::new("list")
                        .about("List historical alarms")
                        .arg(
                            Arg::new("output")
                                .long("output")
                                .short('o')
                                .value_parser(["short", "wide", "detailed"])
                                .default_value("short")
                                .help("Change the output format"),
                        )
                        .arg(
                            Arg::new("limit")
                                .long("limit")
                                .short('l')
                                .value_parser(clap::value_parser!(usize))
                                .default_value("30")
                                .help("Maximum number of alarm history entries to fetch"),
                        )
                        .arg(Arg::new("since").long("since").value_name("TIME").help(
                            "Show alarms since this time (e.g., 2h, 30m, 2025-11-14T09:00:00Z)",
                        ))
                        .arg(Arg::new("until").long("until").value_name("TIME").help(
                            "Show alarms until this time (e.g., 2h, 30m, 2025-11-14T09:00:00Z)",
                        )),
                ),
        )
        .subcommand(
            Command::new("clear").about("Clear active alarms").arg(
                Arg::new("id")
                    .required(true)
                    .num_args(1..)
                    .help("The alarm IDs to clear"),
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
        Some(("history", sub_args)) => run_history(sub_args),
        Some(("clear", sub_args)) => clear(sub_args),
        _ => {
            eprintln!("No subcommand provided");
            std::process::exit(1);
        }
    }
}

fn run_history(args: &ArgMatches) {
    match args.subcommand() {
        Some(("list", sub_args)) => {
            let limit = *sub_args.get_one::<usize>("limit").unwrap();
            let from_date = sub_args.get_one::<String>("since").map(|s| {
                parse_time_filter(s).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                })
            });
            let to_date = sub_args.get_one::<String>("until").map(|s| {
                parse_time_filter(s).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                })
            });
            match sub_args.get_one::<String>("output").map(|s| s.as_str()) {
                Some("wide") => history_list_wide(limit, from_date, to_date),
                Some("detailed") => history_list_detailed(limit, from_date, to_date),
                _ => history_list(limit, from_date, to_date),
            }
        }
        None => {
            let limit = *args.get_one::<usize>("limit").unwrap();
            let from_date = args.get_one::<String>("since").map(|s| {
                parse_time_filter(s).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                })
            });
            let to_date = args.get_one::<String>("until").map(|s| {
                parse_time_filter(s).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                })
            });
            match args.get_one::<String>("output").map(|s| s.as_str()) {
                Some("wide") => history_list_wide(limit, from_date, to_date),
                Some("detailed") => history_list_detailed(limit, from_date, to_date),
                _ => history_list(limit, from_date, to_date),
            }
        }
        _ => {
            eprintln!("Unknown subcommand");
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
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_id
                .as_ref()
                .and_then(|id| output_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("output: {}", s)),
            alarm
                .affected_input
                .as_ref()
                .and_then(|id| input_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .affected_output
                .as_ref()
                .and_then(|id| output_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("output: {}", s)),
            alarm
                .input_name
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_name
                .as_deref()
                .filter(|s| !s.is_empty())
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
    let ports = client
        .list_ports_by_ids(
            alarms
                .iter()
                .filter_map(|a| a.physical_port_id.as_ref())
                .cloned()
                .collect(),
        )
        .expect("Failed to list ports");

    let input_map: HashMap<String, String> = inputs
        .into_iter()
        .map(|input| (input.id, input.name))
        .collect();
    let output_map: HashMap<String, String> = outputs
        .into_iter()
        .map(|output| (output.id, output.name))
        .collect();
    let port_map: HashMap<String, String> =
        ports.into_iter().map(|port| (port.id, port.name)).collect();

    let mut builder = Builder::default();
    builder.push_record([
        "Time Ago",
        "Severity",
        "ID",
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
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_id
                .as_ref()
                .and_then(|id| output_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("output: {}", s)),
            alarm
                .affected_input
                .as_ref()
                .and_then(|id| input_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .affected_output
                .as_ref()
                .and_then(|id| output_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("output: {}", s)),
            alarm
                .input_name
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_name
                .as_deref()
                .filter(|s| !s.is_empty())
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

        let port = alarm
            .physical_port_id
            .as_ref()
            .and_then(|port_id| port_map.get(port_id))
            .map(|s| s.as_str())
            .unwrap_or("-");

        builder.push_record([
            &time_ago,
            &alarm.alarm_severity,
            &alarm.alarm_id,
            &alarm.alarm_cause,
            alarm.text.as_deref().unwrap_or("-"),
            alarm.appliance_name.as_deref().unwrap_or("-"),
            &entity,
            &alarm.alarm_type,
            &alarm.object_name,
            alarm.object_purpose.as_deref().unwrap_or("-"),
            port,
            &alarm.repeat_count.to_string(),
            alarm.region.as_deref().unwrap_or("-"),
            alarm.raised_at.as_deref().unwrap_or("-"),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}

fn history_list(limit: usize, from_date: Option<String>, to_date: Option<String>) {
    let client = new_client();
    let alarms = client
        .list_alarm_history(limit, from_date, to_date)
        .expect("Failed to list alarm history");

    if alarms.is_empty() {
        println!("No alarm history found");
        return;
    }

    let input_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.input_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let input_ids: Vec<String> = Vec::from_iter(input_ids);

    let output_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.output_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let output_ids: Vec<String> = Vec::from_iter(output_ids);

    let mut inputs = Vec::new();
    for chunk in input_ids.chunks(50) {
        inputs.extend(
            client
                .list_inputs_by_ids(chunk.to_vec())
                .expect("Failed to list inputs"),
        );
    }

    let mut outputs = Vec::new();
    for chunk in output_ids.chunks(50) {
        outputs.extend(
            client
                .list_outputs_by_ids(chunk.to_vec())
                .expect("Failed to list outputs"),
        );
    }

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
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_id
                .as_ref()
                .and_then(|id| output_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("output: {}", s)),
            alarm
                .input_name
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_name
                .as_deref()
                .filter(|s| !s.is_empty())
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
        let message = if message.len() > 50 {
            format!("{}…", &message[..49])
        } else {
            message.to_string()
        };

        builder.push_record([
            &time_ago,
            &alarm.alarm_severity,
            &alarm.alarm_cause,
            &message,
            appliance,
            &entity,
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}

fn history_list_wide(limit: usize, from_date: Option<String>, to_date: Option<String>) {
    let client = new_client();
    let alarms = client
        .list_alarm_history(limit, from_date, to_date)
        .expect("Failed to list alarm history");

    if alarms.is_empty() {
        println!("No alarm history found");
        return;
    }

    let input_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.input_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let input_ids: Vec<String> = Vec::from_iter(input_ids);

    let output_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.output_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let output_ids: Vec<String> = Vec::from_iter(output_ids);

    let port_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.physical_port_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let port_ids: Vec<String> = Vec::from_iter(port_ids);

    let mut inputs = Vec::new();
    for chunk in input_ids.chunks(50) {
        inputs.extend(
            client
                .list_inputs_by_ids(chunk.to_vec())
                .expect("Failed to list inputs"),
        );
    }

    let mut outputs = Vec::new();
    for chunk in output_ids.chunks(50) {
        outputs.extend(
            client
                .list_outputs_by_ids(chunk.to_vec())
                .expect("Failed to list outputs"),
        );
    }

    let mut ports = Vec::new();
    for chunk in port_ids.chunks(50) {
        ports.extend(
            client
                .list_ports_by_ids(chunk.to_vec())
                .expect("Failed to list ports"),
        );
    }

    let input_map: HashMap<String, String> = inputs
        .into_iter()
        .map(|input| (input.id, input.name))
        .collect();
    let output_map: HashMap<String, String> = outputs
        .into_iter()
        .map(|output| (output.id, output.name))
        .collect();
    let port_map: HashMap<String, String> =
        ports.into_iter().map(|port| (port.id, port.name)).collect();

    let mut builder = Builder::default();
    builder.push_record([
        "Time Ago",
        "ID",
        "Severity",
        "Cause",
        "Message",
        "Appliance",
        "Entity",
        "Type",
        "Object Name",
        "Port",
        "Raised At",
        "Cleared At",
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
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_id
                .as_ref()
                .and_then(|id| output_map.get(id))
                .filter(|s| !s.is_empty())
                .map(|s| format!("output: {}", s)),
            alarm
                .input_name
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| format!("input: {}", s)),
            alarm
                .output_name
                .as_deref()
                .filter(|s| !s.is_empty())
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
            .and_then(|port_id| port_map.get(port_id))
            .map(|s| s.as_str())
            .unwrap_or("-");

        let message = alarm.text.as_deref().unwrap_or("-");
        let message = if message.len() > 50 {
            format!("{}…", &message[..49])
        } else {
            message.to_string()
        };

        builder.push_record([
            &time_ago,
            &alarm.alarm_id,
            &alarm.alarm_severity,
            &alarm.alarm_cause,
            &message,
            alarm.appliance_name.as_deref().unwrap_or("-"),
            &entity,
            &alarm.alarm_type,
            &alarm.object_name,
            port,
            alarm.raised_at.as_deref().unwrap_or("-"),
            alarm.cleared_at.as_deref().unwrap_or("-"),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}

fn history_list_detailed(limit: usize, from_date: Option<String>, to_date: Option<String>) {
    let client = new_client();
    let alarms = client
        .list_alarm_history(limit, from_date, to_date)
        .expect("Failed to list alarm history");

    if alarms.is_empty() {
        println!("No alarm history found");
        return;
    }

    let input_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.input_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let input_ids: Vec<String> = Vec::from_iter(input_ids);

    let output_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.output_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let output_ids: Vec<String> = Vec::from_iter(output_ids);

    let port_ids: HashSet<String> = alarms
        .iter()
        .filter_map(|a| a.physical_port_id.as_ref())
        .filter(|id| !id.is_empty())
        .cloned()
        .collect();
    let port_ids: Vec<String> = Vec::from_iter(port_ids);

    let mut inputs = Vec::new();
    for chunk in input_ids.chunks(50) {
        inputs.extend(
            client
                .list_inputs_by_ids(chunk.to_vec())
                .expect("Failed to list inputs"),
        );
    }

    let mut outputs = Vec::new();
    for chunk in output_ids.chunks(50) {
        outputs.extend(
            client
                .list_outputs_by_ids(chunk.to_vec())
                .expect("Failed to list outputs"),
        );
    }

    let mut ports = Vec::new();
    for chunk in port_ids.chunks(50) {
        ports.extend(
            client
                .list_ports_by_ids(chunk.to_vec())
                .expect("Failed to list ports"),
        );
    }

    let input_map: HashMap<String, String> = inputs
        .into_iter()
        .map(|input| (input.id, input.name))
        .collect();
    let output_map: HashMap<String, String> = outputs
        .into_iter()
        .map(|output| (output.id, output.name))
        .collect();
    let port_map: HashMap<String, String> =
        ports.into_iter().map(|port| (port.id, port.name)).collect();

    let mut first = true;
    for alarm in alarms.iter() {
        if !first {
            println!();
        }
        first = false;
        println!("ID:               {}", alarm.alarm_id);
        println!("Severity:         {}", alarm.alarm_severity);
        println!("Cause:            {}", alarm.alarm_cause);
        println!("Type:             {}", alarm.alarm_type);
        println!("Object name:      {}", alarm.object_name);

        if let Some(text) = &alarm.text {
            println!("Message:          {}", text);
        }

        if let Some(appliance) = &alarm.appliance_name.as_ref().filter(|a| !a.is_empty()) {
            println!("Appliance:        {}", appliance);
        }

        if let Some(region) = &alarm.region.as_ref().filter(|r| !r.is_empty()) {
            println!("Region:           {}", region);
        }

        if let Some(purpose) = &alarm.object_purpose.as_ref().filter(|p| !p.is_empty()) {
            println!("Object purpose:   {}", purpose);
        }

        if let Some(input) = &alarm
            .input_id
            .as_ref()
            .filter(|id| !id.is_empty())
            .and_then(|id| input_map.get(id))
        {
            println!("Input:            {}", input);
        }

        if let Some(output) = &alarm
            .output_id
            .as_ref()
            .filter(|id| !id.is_empty())
            .and_then(|id| output_map.get(id))
        {
            println!("Output:           {}", output);
        }

        if let Some(port) = &alarm
            .physical_port_id
            .as_ref()
            .filter(|a| !a.is_empty())
            .and_then(|id| port_map.get(id))
        {
            println!("Port:             {}", port);
        }

        if let Some(raised_at) = &alarm.raised_at.as_ref().filter(|r| !r.is_empty()) {
            println!("Raised:           {}", raised_at);
        }

        if let Some(cleared_at) = &alarm.cleared_at.as_ref().filter(|c| !c.is_empty()) {
            println!("Cleared:          {}", cleared_at);
        }

        if alarm.repeat_count > 0 {
            println!("Repeated:         {} times", alarm.repeat_count);
        }
    }
}

fn clear(args: &ArgMatches) {
    let ids: Vec<&String> = args.get_many::<String>("id").unwrap().collect();
    let client = new_client();

    for id in ids {
        if let Err(e) = client.clear_alarm(id) {
            eprintln!("Failed to clear alarm '{}': {}", id, e);
            std::process::exit(1);
        }
        println!("Cleared alarm '{}'", id);
    }
}
