use std::process;

use tabled::{builder::Builder, settings::Style};

use crate::edge::EdgeClient;

pub fn list(client: EdgeClient) {
    let groups = client.list_groups().expect("Failed to fetch group list");

    let mut builder = Builder::default();

    builder.push_record(["ID", "Name"]);
    for group in groups {
        builder.push_record([group.id, group.name])
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

pub fn show(client: EdgeClient, name: &str) {
    let groups = client.find_groups(name).expect("Failed to find groups");
    if groups.is_empty() {
        println!("No such group: {}", name);
        process::exit(1);
    }
    for group in groups {
        println!("Name:                 {}", group.name);
        println!("ID:                   {}", group.id);
        println!(
            "Appliance secret:     {}",
            group.appliance_secret.unwrap_or("".to_owned())
        );
    }
}
