use crate::config::Config;
use clap::{Arg, ArgMatches, Command};
use clap_complete::{ArgValueCompleter, CompletionCandidate};
use std::ffi::OsStr;
use std::process;
use tabled::builder::Builder;
use tabled::settings::Style;

fn context_name_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let config = Config::load();
    let current_str = current.to_str().unwrap_or("");
    config
        .contexts
        .keys()
        .filter(|name| name.starts_with(current_str))
        .map(|name| CompletionCandidate::new(name.clone()))
        .collect()
}

pub fn subcommand() -> Command {
    Command::new("context")
        .about("Manage Edge installation contexts")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("List all contexts"))
        .subcommand(Command::new("current").about("Display the current context"))
        .subcommand(
            Command::new("use")
                .about("Switch to a different context")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the context to use")
                        .add(ArgValueCompleter::new(context_name_completer)),
                ),
        )
        .subcommand(
            Command::new("delete").about("Delete a context").arg(
                Arg::new("name")
                    .required(true)
                    .help("The name of the context to delete")
                    .add(ArgValueCompleter::new(context_name_completer)),
            ),
        )
}

pub fn run(args: &ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("current", _)) => current(),
        Some(("use", args)) => {
            use_context(args.get_one::<String>("name").expect("name is required"))
        }
        Some(("delete", args)) => delete(args.get_one::<String>("name").expect("name is required")),
        _ => {
            eprintln!("Unknown subcommand. Use --help for usage information.");
            process::exit(1);
        }
    }
}

fn list() {
    let config = Config::load();

    if config.contexts.is_empty() {
        eprintln!("No contexts found. Run 'edgectl login' to create one.");
        process::exit(1);
    }

    let mut builder = Builder::default();
    builder.push_record(["Current", "Name", "URL", "Username"]);

    for (name, context) in config.list_contexts() {
        let is_current = config.context.as_ref() == Some(name);
        let current_marker = if is_current { "*" } else { "" };
        builder.push_record([current_marker, name, &context.url, &context.username]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}

fn current() {
    let config = Config::load();

    match &config.context {
        Some(name) => println!("{}", name),
        None => {
            eprintln!("No current context set. Run 'edgectl login' to create one.");
            process::exit(1);
        }
    }
}

fn use_context(name: &str) {
    let mut config = Config::load();

    if !config.contexts.contains_key(name) {
        eprintln!("Context '{}' not found.", name);
        process::exit(1);
    }

    if let Err(e) = config.set_current_context(name.to_owned()) {
        eprintln!("Failed to set current context: {}", e);
        process::exit(1);
    }

    if let Err(e) = config.save() {
        eprintln!("Failed to save config: {}", e);
        process::exit(1);
    }
}

fn delete(name: &str) {
    let mut config = Config::load();

    if !config.contexts.contains_key(name) {
        eprintln!("Context '{}' not found.", name);
        process::exit(1);
    }

    if let Err(e) = config.delete_context(name) {
        eprintln!("Failed to delete context: {}", e);
        process::exit(1);
    }

    if let Err(e) = config.save() {
        eprintln!("Failed to save config: {}", e);
        process::exit(1);
    }

    println!("Deleted context '{}'", name);
}
