use crate::config::Config;
use crate::edge::{ApiTokenInit, EdgeClient};
use chrono::{Duration, Utc};
use clap::{ArgMatches, Command};
use std::env;
use std::io::{self, IsTerminal, Write};

pub fn subcommand() -> Command {
    Command::new("login")
        .about("Login and create an API token")
        .subcommand(Command::new("status").about("Show current user information"))
}

pub fn run(args: &ArgMatches) {
    match args.subcommand() {
        Some(("status", _)) => status(),
        None => login(),
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

fn login() {
    let (url, username, password) = if io::stdin().is_terminal() {
        let conf = Config::load();
        let url = prompt_or_env("URL", env::var("EDGE_URL").ok().or(conf.url));
        let username = prompt_or_env(
            "Username",
            Some(
                env::var("EDGE_USER")
                    .ok()
                    .or(conf.username)
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

        (url, username, password)
    } else {
        // Non-interactive mode: read from env vars only
        let url = env::var("EDGE_URL").unwrap_or_else(|_| {
            eprintln!("Error: EDGE_URL is required in non-interactive mode");
            std::process::exit(1);
        });

        let username = env::var("EDGE_USER").unwrap_or_else(|_| "admin".to_string());

        let password = env::var("EDGE_PASSWORD").unwrap_or_else(|_| {
            eprintln!("Error: EDGE_PASSWORD is required in non-interactive mode");
            std::process::exit(1);
        });

        (url, username, password)
    };

    let client = EdgeClient::with_url(&url);

    let user = match client.login(username.clone(), password) {
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

    let config = Config {
        url: Some(url),
        token: token.token,
        token_name: Some(token_name),
        username: Some(user.username),
    };

    if let Err(e) = config.save() {
        eprintln!("Failed to save configuration: {}", e);
        std::process::exit(1);
    }

    println!("Logged in as {}", username);
}

fn status() {
    let config = Config::load();

    println!(
        "Username: {}",
        config.username.unwrap_or("unknown".to_owned())
    );
    println!("URL: {}", config.url.unwrap_or("unknown".to_owned()));
}
