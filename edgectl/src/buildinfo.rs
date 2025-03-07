use std::{fmt, process};

use crate::edge::{EdgeClient, Product};

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NimbraEdge => f.pad("Nimbra Edge"),
            Self::ConnectIt => f.pad("Connect iT"),
        }
    }
}

pub fn show(client: EdgeClient) {
    let info = match client.get_build_info() {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Failed to get build info: {}", e);
            process::exit(1);
        }
    };

    println!("Release:      {}", info.release);
    println!("Build time:   {}", info.build_time);
    println!("Pipeline:     {}", info.pipeline);
    println!("Commit:       {}", info.commit);
    println!("Product:      {}", info.product);
}
