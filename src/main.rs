extern crate clap;
extern crate reqwest;

use std::process::Command;
use std::io::prelude::*;
use std::fs::File;

use clap::{App, Arg, SubCommand, AppSettings};
use serde_json::json;

struct Page {
    title: String,
    space: String,
    source_file: String,
    version: u32,
}

struct Credentials {
    username: String,
    password: String,
    endpoint: String,
}

fn get_content(page: Page) -> String {
    let mut file = File::open(page.source_file).expect("unable to open file");
    let mut page_body = String::new();
    file.read_to_string(&mut page_body).expect("unable to read file");
    let content = json!({
        "version": {
            "number": page.version 
        },
        "type": "page",
        "space": {
            "key": page.space
        },
        "title": page.title,
        "body": {
            "storage": {
                "value": page_body,
                "representation": "wiki"
            }
        }
    });
    content.to_string()
}

fn get_password(secret_name: String) -> String {
    let output = Command::new("pass")
        .arg(secret_name)
        .output()
        .expect("fail pass");
    let lines = String::from_utf8(output.stdout).expect("failage");
    let v: Vec<&str> = lines.trim().split('\n').collect();
    v[0].to_string()
}

fn create(credentials: Credentials, content: String) -> reqwest::Response {
    let client = reqwest::Client::new();
    client.post(credentials.endpoint.as_str())
        .basic_auth(credentials.username, Some(credentials.password))
        .body(content)
        .header("Content-Type", "application/json")
        .send()
        .expect("post fail")
}

fn update(credentials: Credentials, content: String, id: u64) -> reqwest::Response {
    let endpoint = format!("{}/{}", credentials.endpoint, id);
    let client = reqwest::Client::new();
    client.put(endpoint.as_str())
        .basic_auth(credentials.username, Some(credentials.password))
        .body(content)
        .header("Content-Type", "application/json")
        .send()
        .expect("post fail")
}

fn main() {
    let matches = App::new("meh")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("create")
            .about("create a page")
            .arg(Arg::with_name("secret")
                .short("s")
                .long("secret")
                .help("a pass secret containing API password")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("username")
                .short("u")
                .long("username")
                .help("username for the API")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("space")
                .short("p")
                .long("space")
                .help("Confluence space")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("title")
                .short("t")
                .long("title")
                .help("title for the page to create")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("source")
                .short("f")
                .long("source")
                .help("source file for the page")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("endpoint")
                .short("e")
                .long("endpoint")
                .help("endpoint for Confluence API")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("update")
            .about("update a page")
            .arg(Arg::with_name("secret")
                .short("s")
                .long("secret")
                .help("a pass secret containing API password")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("username")
                .short("u")
                .long("username")
                .help("username for the API")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("space")
                .short("p")
                .long("space")
                .help("Confluence space")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("title")
                .short("t")
                .long("title")
                .help("title for the page to create")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("source")
                .short("f")
                .long("source")
                .help("source file for the page")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("endpoint")
                .short("e")
                .long("endpoint")
                .help("endpoint for Confluence API")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("id")
                .short("i")
                .long("id")
                .help("ID for the page to update")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("version")
                .short("v")
                .long("version")
                .help("version to set for the updated page")
                .takes_value(true)
                .required(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let pass_secret = matches.value_of("secret").unwrap();
        let space = matches.value_of("space").unwrap();
        let title = matches.value_of("title").unwrap();
        let source = matches.value_of("source").unwrap();
        let username = matches.value_of("username").unwrap();
        let endpoint = matches.value_of("endpoint").unwrap();

        let password = get_password(pass_secret.to_string());
        let credentials = Credentials{username: username.to_string(), password: password, endpoint: endpoint.to_string()};
        let page = Page{title: title.to_string(), space: space.to_string(), source_file: source.to_string(), version: 1};
        let content = get_content(page);
        let mut response = create(credentials, content);
        println!("{}", response.text().expect("response text fail"));
        println!("{}", response.status());
    } else if let Some(matches) = matches.subcommand_matches("update") {
        let pass_secret = matches.value_of("secret").unwrap();
        let space = matches.value_of("space").unwrap();
        let title = matches.value_of("title").unwrap();
        let source = matches.value_of("source").unwrap();
        let username = matches.value_of("username").unwrap();
        let endpoint = matches.value_of("endpoint").unwrap();
        let version: u32 = matches.value_of("version").unwrap().parse().expect("failed parsing int");
        let id: u64 = matches.value_of("id").unwrap().parse().expect("failed parsing int");

        let password = get_password(pass_secret.to_string());
        let credentials = Credentials{username: username.to_string(), password: password, endpoint: endpoint.to_string()};
        let page = Page{title: title.to_string(), space: space.to_string(), source_file: source.to_string(), version: version};
        let content = get_content(page);
        let mut response = update(credentials, content, id);
        println!("{}", response.text().expect("response text fail"));
        println!("{}", response.status());
    }
}
