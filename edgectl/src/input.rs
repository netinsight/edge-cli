use std::collections::BTreeMap;
use std::fmt;
use std::process;

use anyhow::{anyhow, Context};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{
    AppliancePhysicalPort, EdgeClient, GeneratorBitrate, GeneratorBitrateCBR, GeneratorInputPort,
    InputAdminStatus, NewInputPort, RtpInputPort, SdiEncoderAudioStream, SdiEncoderSettings,
    SdiInputPort, ThumbnailMode, UdpInputPort,
};

impl fmt::Display for crate::edge::InputHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.state == "allOk" {
            write!(f, "\x1b[32m✓\x1b[0m")
        } else {
            write!(f, "\x1b[31m✗\x1b[0m {}", self.title)
        }
    }
}

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
            input.admin_status.to_string(),
            input.buffer_size.to_string(),
            input.preview_settings.mode,
            input.thumbnail_mode.to_string(),
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

pub fn show(client: EdgeClient, name: &str) {
    let inputs = client.find_inputs(name);
    let inputs = match inputs {
        Ok(inputs) => inputs,
        Err(e) => {
            println!("Failed to find inputs: {:?}", e);
            process::exit(1);
        }
    };

    for input in inputs {
        let group = client.get_group(&input.owner);
        let group_name = group.map(|g| g.name).unwrap_or("unknown".to_owned());

        println!("ID:             {}", input.id);
        println!("Name:           {}", input.name);
        println!("Admin status:   {}", input.admin_status);
        println!("Owner:          {}", group_name);
        println!("Buffer:         {}", input.buffer_size);
        println!("Preview:        {}", input.preview_settings.mode);
        println!("Thumbnail mode: {}", input.thumbnail_mode);
        println!("TR 101 290:     {}", input.tr101290_enabled);
        println!("Can subscribe:  {}", input.can_subscribe);
        println!(
            "Appliances:     {}",
            input
                .appliances
                .into_iter()
                .map(|a| a.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
        if let Some(ports) = input.ports {
            println!("Ports:");
            for port in ports {
                let port_details = client.get_port(&port.physical_port);
                let name = port_details.map(|p| p.name).unwrap_or("unknown".to_owned());
                println!("  - Mode:                   {}", port.mode);
                println!("    Source interface:       {}", name);
                println!("    Copies:                 {}", port.copies);
            }
        }
        println!("Created:        {}", input.created_at);
        println!("Updated:        {}", input.updated_at);
        println!("Health:         {}", input.health);
    }
}

pub struct NewInput {
    pub name: String,
    pub thumbnails: bool,
    pub mode: NewInputMode,
}

pub enum NewInputMode {
    Rtp(NewRtpInputMode),
    Udp(NewUdpInputMode),
    Sdi(NewSdiInputMode),
    Generator(NewGeneratorInputMode),
}

pub struct NewRtpInputMode {
    pub appliance: String,
    pub interface: String,
    pub port: u16,
    pub fec: bool,
    pub multicast_address: Option<String>,
}
pub struct NewUdpInputMode {
    pub appliance: String,
    pub interface: String,
    pub port: u16,
    pub multicast_address: Option<String>,
}
pub struct NewSdiInputMode {
    pub appliance: String,
    pub interface: String,
}

pub struct NewGeneratorInputMode {
    pub appliance: String,
    pub bitrate: Bitrate,
}

#[derive(Clone)]
pub enum Bitrate {
    Vbr,
    Cbr(u64),
}

pub fn create(client: EdgeClient, new_input: NewInput) {
    let ports = match new_input.mode {
        NewInputMode::Rtp(rtp) => {
            let interface = get_physical_port(&client, &rtp.appliance, &rtp.interface);
            vec![NewInputPort::Rtp(RtpInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                address: interface
                    .addresses
                    .first()
                    .expect("Expected at least one address on the appliance physical port")
                    .address
                    .to_owned(),
                port: rtp.port,
                fec: rtp.fec,
                multicast_address: rtp.multicast_address,
            })]
        }
        NewInputMode::Udp(udp) => {
            let interface = get_physical_port(&client, &udp.appliance, &udp.interface);
            vec![NewInputPort::Udp(UdpInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                address: interface
                    .addresses
                    .first()
                    .expect("Expected at least one address on the appliance physical port")
                    .address
                    .to_owned(),
                port: udp.port,
                multicast_address: udp.multicast_address,
            })]
        }
        NewInputMode::Sdi(sdi) => {
            let interface = get_physical_port(&client, &sdi.appliance, &sdi.interface);
            vec![NewInputPort::Sdi(SdiInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                encoder_settings: SdiEncoderSettings {
                    video_codec: "h.264".to_owned(),
                    total_bitrate: 15000000,
                    gop_size_frames: 150,
                    audio_streams: vec![SdiEncoderAudioStream {
                        codec: "aes3".to_owned(),
                        pair: 1,
                        bitrate: 1920,
                        kind: "stereo".to_owned(),
                    }],
                },
            })]
        }
        NewInputMode::Generator(generator) => {
            let interface = get_physical_port(&client, &generator.appliance, "lo");
            vec![NewInputPort::Generator(GeneratorInputPort {
                copies: 1,
                physical_port: interface.id.to_owned(),
                bitrate: match generator.bitrate {
                    Bitrate::Vbr => GeneratorBitrate::Vbr,
                    Bitrate::Cbr(bitrate) => GeneratorBitrate::Cbr(GeneratorBitrateCBR { bitrate }),
                },
            })]
        }
    };

    if let Err(e) = client.create_input(crate::edge::NewInput {
        name: new_input.name,
        tr101290_enabled: true,
        broadcast_standard: "dvb".to_owned(),
        thumbnail_mode: if new_input.thumbnails {
            ThumbnailMode::Core
        } else {
            ThumbnailMode::None
        },
        video_preview_mode: if new_input.thumbnails {
            "on demand".to_owned()
        } else {
            "off".to_owned()
        },
        admin_status: InputAdminStatus::On,
        ports,
        buffer_size: 6_000,
        max_bitrate: None,
    }) {
        eprintln!("Failed to create input: {}", e);
        process::exit(1);
    }
}

fn get_physical_port(
    client: &EdgeClient,
    appliance: &str,
    interface: &str,
) -> AppliancePhysicalPort {
    let appl = match client.find_appliances(appliance) {
        Ok(appls) if appls.is_empty() => {
            println!("Could not find appliance {}", appliance);
            process::exit(1);
        }
        Ok(appls) if appls.len() > 1 => {
            println!(
                "Found more than one appliance matching {}: {}",
                appliance,
                appls
                    .into_iter()
                    .map(|a| a.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            process::exit(1);
        }
        Ok(mut appls) => appls.pop().unwrap(),
        Err(e) => {
            println!("Failed to find appliance {}: {}", appliance, e);
            process::exit(1);
        }
    };
    match appl
        .physical_ports
        .into_iter()
        .find(|p| p.name == interface)
    {
        Some(interface) => interface,
        None => {
            println!(
                "Failed to find interface {} on appliance {}",
                interface, appl.name
            );
            process::exit(1);
        }
    }
}

pub fn delete(client: &EdgeClient, name: &str) -> anyhow::Result<()> {
    let inputs = client.find_inputs(name).context("Failed to list inputs")?;
    if inputs.is_empty() {
        return Err(anyhow!("Input not found"));
    }
    for input in inputs {
        client
            .delete_input(&input.id)
            .context("Failed to delete input")?;
        println!("Deleted input {}", input.name);
    }

    Ok(())
}
