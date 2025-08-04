use std::process;

use clap::{ArgMatches, Command};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{new_client, EdgeClient, OutputAdminStatus, OutputHealthState};

pub(crate) fn subcommand() -> clap::Command {
    Command::new("health").about("Check health status of inputs and outputs")
}

pub(crate) fn run(_subcmd: &ArgMatches) {
    let client = new_client();
    let exit_code = check_health(client);
    process::exit(exit_code);
}

fn check_health(client: EdgeClient) -> i32 {
    let mut exit_code = 0;
    let mut unhealthy_inputs = Vec::new();
    let mut unhealthy_outputs = Vec::new();

    // Check input health
    match client.list_inputs() {
        Ok(inputs) => {
            for input in inputs {
                if input.health.state != "allOk" {
                    unhealthy_inputs.push((input.name, format!("{}", input.health)));
                    exit_code = 1;
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to list inputs: {}", e);
            return 1;
        }
    }

    // Check output health
    match client.list_outputs() {
        Ok(outputs) => {
            for output in outputs {
                match output.admin_status {
                    OutputAdminStatus::On => {
                        if let Some(health) = &output.health {
                            if !matches!(health.state, OutputHealthState::AllOk) {
                                let health_msg = if health.title.is_empty() {
                                    health.state.to_string()
                                } else {
                                    format!("{} ({})", health.state, health.title)
                                };
                                unhealthy_outputs.push((output.name, health_msg));
                                exit_code = 1;
                            }
                        }
                    }
                    OutputAdminStatus::Off => {
                        // Skip disabled outputs - they're not considered unhealthy
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to list outputs: {}", e);
            return 1;
        }
    }

    // Display results
    if !unhealthy_inputs.is_empty() {
        println!("Found unhealthy inputs:");
        let mut builder = Builder::default();
        for (name, status) in unhealthy_inputs {
            builder.push_record([format!("  {}", name), status]);
        }
        let mut table = builder.build();
        table.with(Style::empty());
        println!("{}", table);
    }

    if !unhealthy_outputs.is_empty() {
        println!("Found unhealthy outputs:");
        let mut builder = Builder::default();
        for (name, status) in unhealthy_outputs {
            builder.push_record([format!("  {}", name), status]);
        }
        let mut table = builder.build();
        table.with(Style::empty());
        println!("{}", table);
    }

    if exit_code == 0 {
        println!("All OK");
    } else {
        println!("Found inputs/outputs with non-ok status");
    }

    exit_code
}
