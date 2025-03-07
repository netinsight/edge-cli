use std::process;

use tabled::{builder::Builder, settings::Style};

use crate::edge::{EdgeClient, Group, NewGroup};

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

pub(crate) fn core_secret(client: EdgeClient, name: &str) {
    let groups = match client.find_groups(name) {
        Ok(groups) => groups,
        Err(e) => {
            println!("Failed to list groups: {}", e);
            process::exit(1);
        }
    };
    let groups: Vec<&Group> = groups.iter().filter(|g| g.name == name).collect();
    if groups.is_empty() {
        println!("Group not found: {}", name);
        process::exit(1);
    }
    for group in groups {
        let secret = client
            .get_group_core_secret(&group.id)
            .expect("Failed to get group secret");
        println!("{}", secret)
    }
}

pub fn create(client: EdgeClient, name: &str) {
    match client.create_group(NewGroup {
        name: name.to_owned(),
        appliance_secret: uuid::Uuid::new_v4().to_string(),
    }) {
        Err(e) => {
            println!("Failed to create group: {}", e);
            process::exit(1);
        }
        Ok(g) => {
            println!(
                "Created group {} with appliance secret {}",
                g.name,
                g.appliance_secret.unwrap_or("".to_owned())
            )
        }
    }
}

pub fn delete(client: EdgeClient, name: &str) {
    let groups = match client.find_groups(name) {
        Ok(groups) => groups,
        Err(e) => {
            println!("Failed to list groups for deletion: {}", e);
            process::exit(1);
        }
    };
    let groups: Vec<&Group> = groups.iter().filter(|g| g.name == name).collect();
    if groups.is_empty() {
        println!("Group not found: {}", name);
        process::exit(1);
    }
    for group in groups {
        if let Err(e) = client.delete_group(&group.id) {
            println!("Failed to delete group {}: {}", group.name, e);
            process::exit(1);
        }
        println!("Deleted groupd {}", group.name)
    }
}
