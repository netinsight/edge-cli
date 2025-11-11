use clap::{parser::ValuesRef, Arg, ArgMatches, Command};
use tabled::{builder::Builder, settings::Style};

use crate::edge::{new_client, NewGroupRecipientList};

pub(crate) fn subcommand() -> clap::Command {
    Command::new("group-list")
        .about("Manage group lists")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("List all group lists"))
        .subcommand(
            Command::new("show")
                .about("Show details of a group list")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the group list"),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Create a new group list")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the new group list"),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("Optional description for the list"),
                )
                .arg(
                    Arg::new("groups")
                        .long("groups")
                        .short('g')
                        .num_args(1..)
                        .help("The groups to add to the list"),
                )
                .arg(
                    Arg::new("group")
                        .long("group")
                        .help("The group list owner group (defaults to 'system')"),
                ),
        )
        .subcommand(
            Command::new("delete").about("Delete a group list").arg(
                Arg::new("name")
                    .required(true)
                    .num_args(1..)
                    .help("The name of the group list to delete"),
            ),
        )
        .subcommand(
            Command::new("add-group")
                .about("Add a group to a group list")
                .arg(
                    Arg::new("list")
                        .required(true)
                        .help("The name of the group list"),
                )
                .arg(
                    Arg::new("group")
                        .required(true)
                        .num_args(1..)
                        .help("The name of the groups to add"),
                ),
        )
        .subcommand(
            Command::new("remove-group")
                .about("Remove a group from a group list")
                .arg(
                    Arg::new("list")
                        .required(true)
                        .help("The name of the group list"),
                )
                .arg(
                    Arg::new("group")
                        .required(true)
                        .num_args(1..)
                        .help("The name of the groups to remove"),
                ),
        )
}

pub(crate) fn run(args: &ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("show", sub_args)) => show(sub_args),
        Some(("create", sub_args)) => create(sub_args),
        Some(("delete", sub_args)) => delete(sub_args),
        Some(("add-group", sub_args)) => add_group(sub_args),
        Some(("remove-group", sub_args)) => remove_group(sub_args),
        _ => {
            eprintln!("No subcommand provided");
            std::process::exit(1);
        }
    }
}

fn list() {
    let client = new_client();
    let lists = client
        .list_group_recipient_lists()
        .expect("Failed to list group lists");

    if lists.is_empty() {
        println!("No group lists found");
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
        .find_group_recipient_lists(name)
        .expect("Failed to find group list");

    if lists.is_empty() {
        eprintln!("Group list '{}' not found", name);
        std::process::exit(1);
    }

    let list = &lists[0];

    let groups = client
        .get_group_list_members(&list.id)
        .expect("Failed to get groups in list");

    println!("ID:          {}", list.id);
    println!("Name:        {}", list.name);
    println!(
        "Description: {}",
        list.description.as_deref().unwrap_or("-")
    );
    println!("Groups:      {}", groups.len());

    if !groups.is_empty() {
        println!("\nGroups in list:");
        let mut builder = Builder::default();
        builder.push_record(["ID", "NAME"]);

        for group in groups {
            builder.push_record([&group.id, &group.name]);
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

    let groups_to_add = if let Some(group_names) = args.get_many::<String>("groups") {
        get_group_ids_by_names(&client, group_names)
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

    let new_list = NewGroupRecipientList {
        name: name.clone(),
        description,
        group: group_id.clone(),
        add_groups: groups_to_add,
    };

    client
        .create_group_recipient_list(new_list)
        .expect("Failed to create group list");
}

fn delete(args: &ArgMatches) {
    let names = args.get_many::<String>("name").unwrap();
    let client = new_client();

    for name in names {
        let lists = client
            .find_group_recipient_lists(name)
            .expect(format!("Failed to find group list {}", name).as_str());

        if lists.is_empty() {
            eprintln!("Group list '{}' not found", name);
            std::process::exit(1);
        }

        for list in lists {
            client
                .delete_group_recipient_list(&list.id)
                .expect("Failed to delete group list");

            println!("Deleted group list '{}'", list.name);
        }
    }
}

fn add_group(args: &ArgMatches) {
    let list_name = args.get_one::<String>("list").unwrap();
    let group_names = args.get_many::<String>("group").unwrap();
    let client = new_client();

    let lists = client
        .find_group_recipient_lists(list_name)
        .expect("Failed to find group list");
    if lists.is_empty() {
        eprintln!("Group list '{}' not found", list_name);
        std::process::exit(1);
    }
    let list = &lists[0];

    let groups = get_group_ids_by_names(&client, group_names);

    client
        .add_group_to_list(&list.id, &list.name, groups)
        .expect("Failed to add group to list");
}

fn remove_group(args: &ArgMatches) {
    let list_name = args.get_one::<String>("list").unwrap();
    let group_names = args.get_many::<String>("group").unwrap();
    let client = new_client();

    let lists = client
        .find_group_recipient_lists(list_name)
        .expect("Failed to find group list");
    if lists.is_empty() {
        eprintln!("Group list '{}' not found", list_name);
        std::process::exit(1);
    }
    let list = &lists[0];

    let groups = get_group_ids_by_names(&client, group_names);

    client
        .remove_group_from_list(&list.id, &list.name, groups)
        .expect("Failed to remove group from list");
}

fn get_group_ids_by_names(
    client: &crate::edge::EdgeClient,
    names: ValuesRef<'_, String>,
) -> Vec<String> {
    names
        .into_iter()
        .map(|group_name| {
            client
                .find_groups(&group_name)
                .expect(format!("Failed to find group {}", group_name).as_str())
        })
        .flat_map(|f| f.iter().map(|g| g.id.clone()).collect::<Vec<_>>())
        .collect::<Vec<_>>()
}
