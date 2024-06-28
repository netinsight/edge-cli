use tabled::{builder::Builder, settings::Style};

use crate::edge::EdgeClient;

pub fn list(client: EdgeClient) {
    let groups = client.list_groups().unwrap();

    let mut builder = Builder::default();

    builder.push_record(["ID", "Name", "Appliance secret"]);
    for group in groups {
        builder.push_record([group.id, group.name, group.appliance_secret])
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}
