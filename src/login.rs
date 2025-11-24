use crate::config::{Config, ContextConfig};
use crate::edge::{ApiTokenInit, EdgeClient};
use chrono::{Duration, Utc};
use clap::{Arg, ArgMatches, Command};
use std::env;
use std::io::{self, IsTerminal, Write};

pub fn subcommand() -> Command {
    Command::new("login")
        .about("Login and create an API token")
        .arg(
            Arg::new("url")
                .required(false)
                .help("The URL of the edge installation"),
        )
        .arg(
            Arg::new("context")
                .long("context")
                .help("Custom name for the context (defaults to hostname from URL)"),
        )
        .subcommand(Command::new("status").about("Show current user information"))
}

pub fn run(args: &ArgMatches) {
    match args.subcommand() {
        Some(("status", _)) => status(),
        None => {
            let url = args.get_one::<String>("url").cloned();
            let context_name = args.get_one::<String>("context").cloned();

            let (url, username, password, context_name) = if io::stdin().is_terminal() {
                let conf = Config::load();
                let context = context_name
                    .as_ref()
                    .and_then(|context_name| conf.contexts.get(context_name));
                let current_url = context.map(|c| c.url.clone());
                let current_username = context.map(|c| c.username.clone());

                let url = url.unwrap_or_else(|| {
                    prompt_or_env("URL", env::var("EDGE_URL").ok().or(current_url))
                });
                let username = prompt_or_env(
                    "Username",
                    Some(
                        env::var("EDGE_USER")
                            .ok()
                            .or(current_username)
                            .unwrap_or("admin".to_owned()),
                    ),
                );

                let password = if let Ok(pwd) = env::var("EDGE_PASSWORD") {
                    pwd
                } else {
                    print!("Password: ");
                    io::stdout().flush().unwrap();
                    match rpassword::read_password() {
                        Ok(password) => password,
                        Err(_) => {
                            let mut password = String::new();
                            io::stdin().read_line(&mut password).unwrap();
                            password.trim().to_string()
                        }
                    }
                };

                let default_context_name = url
                    .strip_prefix("https://")
                    .or_else(|| url.strip_prefix("http://"))
                    .unwrap_or(&url)
                    .split('/')
                    .next()
                    .unwrap_or("default")
                    .to_string();

                let context_name = context_name
                    .unwrap_or_else(|| prompt_or_env("Context name", Some(default_context_name)));

                (url, username, password, context_name)
            } else {
                // Non-interactive mode: read from env vars only
                let url = url.or(env::var("EDGE_URL").ok()).unwrap_or_else(|| {
                    eprintln!("Error: EDGE_URL is required in non-interactive mode");
                    std::process::exit(1);
                });

                let username = env::var("EDGE_USER").unwrap_or_else(|_| "admin".to_string());

                let password = env::var("EDGE_PASSWORD").unwrap_or_else(|_| {
                    eprintln!("Error: EDGE_PASSWORD is required in non-interactive mode");
                    std::process::exit(1);
                });

                (url, username, password, "default".to_owned())
            };

            login(&url, &username, &password, &context_name)
        }
        _ => {
            eprintln!("Unknown subcommand");
            std::process::exit(1);
        }
    }
}

fn prompt_or_env(prompt: &str, default: Option<String>) -> String {
    print!("{}", prompt);
    if let Some(ref def) = default {
        print!(" [{}]", def);
    }
    print!(": ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        if let Some(def) = default {
            return def.to_string();
        }
    }

    input.to_string()
}

fn login(url: &str, username: &str, password: &str, context_name: &str) {
    let client = EdgeClient::with_url(url);

    let user = match client.login(username.to_owned(), password.to_owned()) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Failed to authenticate: {}", e);
            std::process::exit(1);
        }
    };

    let hostname = env::var("HOSTNAME").ok().unwrap_or("unknown".to_owned());
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let token_name = format!("edgectl-{}-{}", hostname, timestamp);
    let expires_at = Utc::now() + Duration::days(90);

    let token_init = ApiTokenInit {
        name: token_name.clone(),
        role: user.role,
        expires_at: expires_at.to_rfc3339(),
        scopes: vec!["api".to_string()],
    };

    let token = match client.create_api_token(token_init) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Failed to create API token: {}", e);
            std::process::exit(1);
        }
    };

    let context = ContextConfig {
        url: url.to_owned(),
        token: token.token.unwrap_or_default(),
        token_name: token_name.clone(),
        username: user.username.clone(),
    };

    let mut config = Config::load();
    config.add_context(context_name.to_owned(), context);

    if let Err(e) = config.save() {
        eprintln!("Failed to save configuration: {}", e);
        std::process::exit(1);
    }

    println!("Logged in as {}", username);
}

fn status() {
    let config = Config::load();

    if let Some(context) = config.get_current_context() {
        println!("Username: {}", context.username);
        println!("URL: {}", context.url);
        if let Some(name) = &config.context {
            println!("Context: {}", name);
        }
    } else {
        println!("Not logged in. Run 'edgectl login' to authenticate.");
    }
}
