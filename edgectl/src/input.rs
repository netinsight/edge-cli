use std::collections::BTreeMap;

use tabled::{builder::Builder, settings::Style};

use crate::edge::EdgeClient;

pub fn list(client: EdgeClient) {
    let inputs = client.list_inputs().unwrap();
    let mut builder = Builder::default();
    builder.push_record(["ID", "Name", "Health"]);

    for input in inputs {
        builder.push_record([
            input.id,
            input.name,
            if input.health.state == "allOk" {
                "\x1b[32m✓\x1b[0m".to_owned()
            } else {
                format!("\x1b[31m✗\x1b[0m {}", input.health.title)
            },
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

pub fn list_wide(client: EdgeClient) {
    let inputs = client.list_inputs().unwrap();
    let mut groups = BTreeMap::new();
    let mut group_list = client.list_groups().unwrap();
    while let Some(group) = group_list.pop() {
        groups.insert(group.id.to_owned(), group);
    }

    let mut builder = Builder::default();
    builder.push_record([
        "ID",
        "Name",
        "Group",
        "Enabled",
        "Buffer",
        "Preview",
        "Thumbnails",
        "TR 101 290",
        "can subscribe",
        "Appliances",
        "Health",
    ]);

    for input in inputs {
        builder.push_record([
            input.id,
            input.name,
            groups
                .get(&input.owner)
                .map(|g| g.name.to_owned())
                .unwrap_or("?".to_owned()),
            (match input.admin_status {
                1 => "on",
                0 => "off",
                _ => "unknown",
            })
            .to_owned(),
            input.buffer_size.to_string(),
            input.preview_settings.mode,
            (match input.thumbnail_mode {
                0 => "none",
                2 => "core",
                _ => "unknown",
            })
            .to_owned(),
            if input.tr101290_enabled {
                "on".to_owned()
            } else {
                "off".to_owned()
            },
            input.can_subscribe.to_string(),
            input
                .appliances
                .into_iter()
                .map(|a| a.name)
                .collect::<Vec<String>>()
                .join(", "),
            if input.health.state == "allOk" {
                "\x1b[32m✓\x1b[0m".to_owned()
            } else {
                format!("\x1b[31m✗\x1b[0m {}", input.health.title)
            },
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}
