use clap::{parser::ValuesRef, Arg, ArgMatches, Command};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{new_client, NewOutputRecipientList};

pub(crate) fn subcommand() -> clap::Command {
    Command::new("output-list")
        .about("Manage output lists")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("List all output lists"))
        .subcommand(
            Command::new("show")
                .about("Show details of an output list")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the output list"),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Create a new output list")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the new output list"),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("Optional description for the list"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .num_args(1..)
                        .help("The outputs to add to the list"),
                )
                .arg(
                    Arg::new("group")
                        .long("group")
                        .short('g')
                        .help("The group for the output list (defaults to 'system')"),
                ),
        )
        .subcommand(
            Command::new("delete").about("Delete an output list").arg(
                Arg::new("name")
                    .required(true)
                    .num_args(1..)
                    .help("The name of the output list to delete"),
            ),
        )
        .subcommand(
            Command::new("add-output")
                .about("Add an output to an output list")
                .arg(
                    Arg::new("list")
                        .required(true)
                        .help("The name of the output list"),
                )
                .arg(
                    Arg::new("output")
                        .required(true)
                        .num_args(1..)
                        .help("The name of the outputs to add"),
                ),
        )
        .subcommand(
            Command::new("remove-output")
                .about("Remove an output from an output list")
                .arg(
                    Arg::new("list")
                        .required(true)
                        .help("The name of the output list"),
                )
                .arg(
                    Arg::new("output")
                        .required(true)
                        .num_args(1..)
                        .help("The name of the outputs to remove"),
                ),
        )
}

pub(crate) fn run(args: &ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("show", sub_args)) => show(sub_args),
        Some(("create", sub_args)) => create(sub_args),
        Some(("delete", sub_args)) => delete(sub_args),
        Some(("add-output", sub_args)) => add_output(sub_args),
        Some(("remove-output", sub_args)) => remove_output(sub_args),
        _ => {
            eprintln!("No subcommand provided");
            std::process::exit(1);
        }
    }
}

fn list() {
    let client = new_client();
    let lists = client
        .list_output_recipient_lists()
        .expect("Failed to list output lists");

    if lists.is_empty() {
        println!("No output lists found");
        return;
    }

    let mut builder = Builder::default();
    builder.push_record(["ID", "Name", "Description"]);

    for list in lists {
        let desc = list.description.as_deref().unwrap_or("-");
        builder.push_record([&list.id, &list.name, desc]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    println!("{}", table);
}

fn show(args: &ArgMatches) {
    let name = args.get_one::<String>("name").unwrap();
    let client = new_client();

    let lists = client
        .find_output_recipient_lists(name)
        .expect("Failed to find output list");

    if lists.is_empty() {
        eprintln!("Output list '{}' not found", name);
        std::process::exit(1);
    }

    let list = &lists[0];

    let outputs = client
        .get_output_list_members(&list.id)
        .expect("Failed to get outputs in list");

    println!("ID:          {}", list.id);
    println!("Name:        {}", list.name);
    println!(
        "Description: {}",
        list.description.as_deref().unwrap_or("-")
    );
    println!("Outputs:     {}", outputs.len());

    if !outputs.is_empty() {
        println!("\nOutputs in list:");
        let mut builder = Builder::default();
        builder.push_record(["ID", "NAME"]);

        for output in outputs {
            builder.push_record([&output.id, &output.name]);
        }

        let mut table = builder.build();
        table.with(Style::empty());
        println!("{}", table);
    }
}

fn create(args: &ArgMatches) {
    let name = args.get_one::<String>("name").unwrap();
    let description = args.get_one::<String>("description").map(|s| s.to_string());
    let group_name = args
        .get_one::<String>("group")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "system".to_string());

    let client = new_client();

    let outputs = if let Some(output_names) = args.get_many::<String>("output") {
        get_output_ids_by_names(&client, output_names)
    } else {
        vec![]
    };

    let groups = client
        .find_groups(&group_name)
        .expect("Failed to find group");

    if groups.is_empty() {
        eprintln!("Group '{}' not found", group_name);
        std::process::exit(1);
    }

    let group_id = &groups[0].id;

    let new_list = NewOutputRecipientList {
        name: name.clone(),
        description,
        group: group_id.clone(),
        add_outputs: outputs,
    };

    client
        .create_output_recipient_list(new_list)
        .expect("Failed to create output list");
}

fn delete(args: &ArgMatches) {
    let names = args.get_many::<String>("name").unwrap();
    let client = new_client();

    for name in names {
        let list = match client.find_output_recipient_lists(name) {
            Ok(lists) if lists.is_empty() => {
                eprintln!("Output list '{}' not found", name);
                std::process::exit(1);
            }
            Ok(lists) if lists.len() > 1 => {
                eprintln!("Multiple output lists found with name '{}'", name);
                std::process::exit(1);
            }
            Ok(mut lists) => lists.pop().unwrap(),
            Err(e) => {
                eprintln!("Failed to find output list {}: {}", name, e);
                std::process::exit(1);
            }
        };

        client
            .delete_output_recipient_list(&list.id)
            .expect("Failed to delete output list");

        println!("Deleted output list '{}'", list.name);
    }
}

fn add_output(args: &ArgMatches) {
    let list_name = args.get_one::<String>("list").unwrap();
    let output_names = args.get_many::<String>("output").unwrap();
    let client = new_client();

    let lists = client
        .find_output_recipient_lists(list_name)
        .expect("Failed to find output list");
    if lists.is_empty() {
        eprintln!("Output list '{}' not found", list_name);
        std::process::exit(1);
    }
    let list = &lists[0];

    let outputs = get_output_ids_by_names(&client, output_names);

    client
        .add_output_to_list(&list.id, &list.name, outputs)
        .expect("Failed to add output to list");
}

fn remove_output(args: &ArgMatches) {
    let list_name = args.get_one::<String>("list").unwrap();
    let output_names = args.get_many::<String>("output").unwrap();
    let client = new_client();

    let lists = client
        .find_output_recipient_lists(list_name)
        .expect("Failed to find output list");
    if lists.is_empty() {
        eprintln!("Output list '{}' not found", list_name);
        std::process::exit(1);
    }
    let list = &lists[0];

    let outputs = get_output_ids_by_names(&client, output_names);

    client
        .remove_output_from_list(&list.id, &list.name, outputs)
        .expect("Failed to remove output from list");
}

fn get_output_ids_by_names(
    client: &crate::edge::EdgeClient,
    names: ValuesRef<'_, String>,
) -> Vec<String> {
    names
        .into_iter()
        .map(|output_name| {
            let output = match client.find_outputs(output_name) {
                Ok(outputs) if outputs.is_empty() => {
                    eprintln!("Output '{}' not found", output_name);
                    std::process::exit(1);
                }
                Ok(outputs) if outputs.len() > 1 => {
                    eprintln!("Multiple outputs found with name '{}'", output_name);
                    std::process::exit(1);
                }
                Ok(mut outputs) => outputs.pop().unwrap(),
                Err(e) => {
                    eprintln!("Failed to find output {}: {}", output_name, e);
                    std::process::exit(1);
                }
            };
            output.id
        })
        .collect::<Vec<_>>()
}
