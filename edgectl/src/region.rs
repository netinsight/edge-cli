use std::{fmt, process};

use tabled::{builder::Builder, settings::Style};

use crate::edge::{EdgeClient, ExternalRegionMode, NewRegion};

impl fmt::Display for ExternalRegionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core => f.pad("core"),
            Self::ExternalK8s => f.pad("external kubernetes"),
            Self::External => f.pad("external"),
        }
    }
}

pub fn list(client: EdgeClient) {
    let regions = match client.list_regions() {
        Ok(regions) => regions,
        Err(e) => {
            eprintln!("Failed to list regions: {}", e);
            process::exit(1);
        }
    };

    let mut builder = Builder::default();
    builder.push_record(["Name", "ID", "is default", "Type"]);
    for region in regions {
        builder.push_record([
            region.name,
            region.id,
            region.default_region.unwrap_or(false).to_string(),
            region.external.to_string(),
        ])
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table)
}

pub fn create(client: EdgeClient, name: &str) {
    if let Err(e) = client.create_region(NewRegion {
        name: name.to_string(),
        external: ExternalRegionMode::External,
    }) {
        eprintln!("Failed to create region {}: {}", name, e);
        process::exit(1);
    }
}

pub fn delete(client: EdgeClient, name: &str) {
    let region = match client.find_region(name) {
        Ok(regions) if regions.is_empty() => {
            eprintln!("No region named {} found", name);
            process::exit(1);
        }
        Ok(mut regions) if regions.len() == 1 => regions.pop().unwrap(),
        Ok(regions) => {
            eprintln!("Found more than one region matching {}:", name);
            for region in regions {
                eprintln!("{}", region.name)
            }
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to list inputs for deleteion: {}", e);
            process::exit(1);
        }
    };
    if let Err(e) = client.delete_region(&region.id) {
        println!("Failed to delete region {}: {}", region.name, e);
        process::exit(1);
    }
    println!("Deleted region {}", region.name);
}
