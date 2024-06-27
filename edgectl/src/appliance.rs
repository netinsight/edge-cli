use std::fmt;

use tabled::{builder::Builder, settings::Style};

use crate::edge::{ApplianceHealthState, EdgeClient};

impl fmt::Display for ApplianceHealthState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Connected => write!(f, "\x1b[32m✓\x1b[0m connected"),
            Self::Missing => write!(f, "\x1b[31m✗\x1b[0m missing"),
            Self::NeverConnected => write!(f, "\x1b[31m✗\x1b[0m never connected"),
        }
    }
}

pub fn list(client: EdgeClient) {
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
