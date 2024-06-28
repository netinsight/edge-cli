// use std::collections::BTreeMap;
use std::fmt;
// use std::process;

use tabled::{builder::Builder, settings::Style};

use crate::edge::{EdgeClient, OutputAdminStatus, OutputHealthState};

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

pub fn list(client: EdgeClient) {
    let outputs = client.list_outputs().expect("Failed to list outputs");
    let mut builder = Builder::default();
    builder.push_record(["ID", "Name", "Health"]);

    for output in outputs {
        let health = match output.admin_status {
            OutputAdminStatus::On => output
                .health
                .map(|h| format!("{} ({})", h.state, h.title))
                .unwrap_or("unknown".to_owned()),
            OutputAdminStatus::Off => "\x1b[37m⏻\x1b[0m Disabled".to_owned(),
        };
        builder.push_record([output.id, output.name, health]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

pub fn list_wide(_client: EdgeClient) {
    todo!()
}

pub fn show(_client: EdgeClient, _name: &str) {
    todo!()
}

pub fn delete(_client: EdgeClient, _name: &str) {
    todo!()
}
