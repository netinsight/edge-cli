use tabled::{builder::Builder, settings::Style};

use crate::edge::EdgeClient;

pub fn list(client: EdgeClient) {
    let tunnels = client.list_tunnels().expect("Failed to fetch tunnel list");

    let mut builder = Builder::default();

    builder.push_record(["ID", "Type", "Client", "Server", "Inputs"]);
    for tunnel in tunnels {
        builder.push_record([
            tunnel.id.to_string(),
            tunnel.r#type.to_string(),
            tunnel.client_name,
            tunnel.server_name,
            tunnel.inputs.len().to_string(),
        ])
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}
