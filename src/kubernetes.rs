use tabled::{builder::Builder, settings::Style};

use crate::edge::EdgeClient;

pub fn list_nodes(client: EdgeClient) {
    let nodes = client
        .list_kubernetes_nodes()
        .expect("Failed to fetch tunnel list");

    let mut builder = Builder::default();

    builder.push_record([
        "Name",
        "Status",
        "Internal IP",
        "External IP",
        "Hostname",
        "Roles",
        "Kubelet version",
        "Region",
        "Region type",
    ]);
    for node in nodes {
        builder.push_record([
            node.name,
            node.status,
            if node.internal_ip.is_empty() {
                "none".to_owned()
            } else {
                node.internal_ip
            },
            if node.external_ip.is_empty() {
                "none".to_owned()
            } else {
                node.external_ip
            },
            node.hostname,
            node.roles
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            node.kubelet_version.unwrap_or("unknown".to_owned()),
            node.region.name,
            node.region.external.to_string(),
        ])
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}
