use crate::config::Config;
use crate::edge;
use clap::{Arg, ArgMatches, Command};
use tabled::builder::Builder;
use tabled::settings::Style;

pub fn subcommand() -> Command {
    Command::new("token")
        .about("Manage API tokens")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("List all API tokens"))
        .subcommand(
            Command::new("delete").about("Delete an API token").arg(
                Arg::new("name")
                    .required(true)
                    .help("The name of the token to delete"),
            ),
        )
}

pub fn run(args: &ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("delete", sub_args)) => delete(sub_args),
        _ => {
            eprintln!("Unknown subcommand");
            std::process::exit(1);
        }
    }
}

fn list() {
    let client = edge::new_client();

    let tokens = client.list_api_tokens().expect("Failed to list API tokens");

    let mut builder = Builder::default();
    builder.push_record(["Name", "Role", "Expires At", "Scopes"]);

    for token in tokens {
        let scopes = token
            .scopes
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",");

        builder.push_record([&token.name, &token.role, &token.expires_at, &scopes]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}

fn delete(args: &ArgMatches) {
    let name = args.get_one::<String>("name").unwrap();
    let client = edge::new_client();

    let tokens = client.list_api_tokens().expect("Failed to list API tokens");

    let token = tokens.iter().find(|t| &t.name == name).unwrap_or_else(|| {
        eprintln!("Token '{}' not found", name);
        std::process::exit(1);
    });

    let config = Config::load();
    if let Some(token_name) = &config.token_name {
        if token_name == name {
            eprintln!("Warning: Deleting currently active token. You will need to login again.");
            Config::delete().ok();
        }
    }

    client
        .delete_api_token(&token.id)
        .expect("Failed to delete API token");

    println!("Deleted token '{}'", name);
}
