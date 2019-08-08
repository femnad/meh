extern crate clap;
extern crate dirs;
extern crate reqwest;
extern crate serde;

use std::process::Command;
use std::io::prelude::*;
use std::fs::File;

use clap::{App, Arg, SubCommand, AppSettings};
use serde_json::json;
use serde::Deserialize;

const DEFAULT_PROFILE: &str = "default";
const PROFILE_FILE_SUFFIX: &str = "meh/meh.yaml";

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

#[derive(Deserialize)]
struct Profile {
    name: String,
    username: String,
    pass_secret: String,
    endpoint: String,
    space: String,
}

#[derive(Deserialize, Clone)]
struct Version {
    number: u32,
}

#[derive(Deserialize, Clone)]
struct ConfluencePage {
    id: String,
    title: String,
    version: Version,
}

#[derive(Deserialize)]
struct SearchResults {
    results: Vec<ConfluencePage>
}

fn get_profile(profile_name: &str) -> Profile {
    let mut config_file = dirs::config_dir().expect("Unable to get config dir");
    config_file.push(PROFILE_FILE_SUFFIX);
    let mut file = File::open(config_file).expect("Unable to open config file");
    let mut profiles = String::new();
    file.read_to_string(&mut profiles).expect("Error reading config file");
    let profiles: Vec<Profile> = serde_yaml::from_str(&profiles).expect("Unable to deserialize config");
    profiles.into_iter()
        .find(|profile| { profile.name == profile_name })
        .expect("Cannot find desired profile")
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

fn get_endpoint(credentials: &Credentials) -> String {
    format!("{}/{}", credentials.endpoint, "confluence/rest/api/content")
}

fn create(credentials: &Credentials, content: String) -> reqwest::Response {
    let client = reqwest::Client::new();
    client.post(get_endpoint(credentials).as_str())
        .basic_auth(&credentials.username, Some(&credentials.password))
        .body(content)
        .header("Content-Type", "application/json")
        .send()
        .expect("post fail")
}

fn update(credentials: &Credentials, content: String, id: u64) -> reqwest::Response {
    let endpoint = format!("{}/{}", get_endpoint(credentials), id);
    let client = reqwest::Client::new();
    client.put(endpoint.as_str())
        .basic_auth(&credentials.username, Some(&credentials.password))
        .body(content)
        .header("Content-Type", "application/json")
        .send()
        .expect("post fail")
}

fn search(credentials: &Credentials, space: String, title: String) -> ConfluencePage {
    let endpoint = format!("{endpoint}?spaceKey={space}&title={title}&expand=version",
        endpoint=get_endpoint(credentials), space=space, title=title);
    let client = reqwest::Client::new();
    let mut response = client.get(endpoint.as_str())
        .basic_auth(&credentials.username, Some(&credentials.password))
        .send()
        .expect("search by title and space fail");
    let search_results: SearchResults = response.json().expect("cannot extract JSON");
    let first_result = &search_results.results[0];
    first_result.clone()
}

fn main() {
    let matches = App::new("meh")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("create")
            .about("create a page")
            .arg(Arg::with_name("profile")
                .short("p")
                .long("profile")
                .help("a profile name")
                .takes_value(true)
                .default_value(&DEFAULT_PROFILE))
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
                .required(true)))
        .subcommand(SubCommand::with_name("update")
            .about("update a page")
            .arg(Arg::with_name("profile")
                .short("p")
                .long("profile")
                .help("a profile name")
                .takes_value(true)
                .default_value(&DEFAULT_PROFILE))
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
        .subcommand(SubCommand::with_name("search")
            .about("search for a page")
            .arg(Arg::with_name("profile")
                .short("p")
                .long("profile")
                .help("a profile name")
                .takes_value(true)
                .default_value(&DEFAULT_PROFILE))
            .arg(Arg::with_name("title")
                .short("t")
                .long("title")
                .help("title for the page to create")
                .takes_value(true)
                .required(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let title = matches.value_of("title").unwrap();
        let source = matches.value_of("source").unwrap();

        let profile_name = matches.value_of("profile").unwrap();
        let profile = get_profile(profile_name);

        let password = get_password(profile.pass_secret.to_string());
        let credentials = Credentials{username: profile.username.to_string(), password: password, endpoint: profile.endpoint};
        let page = Page{title: title.to_string(), space: profile.space, source_file: source.to_string(), version: 1};
        let content = get_content(page);

        let mut response = create(&credentials, content);
        println!("{}", response.text().expect("response text fail"));
        println!("{}", response.status());
    } else if let Some(matches) = matches.subcommand_matches("update") {
        let title = matches.value_of("title").unwrap();
        let source = matches.value_of("source").unwrap();

        let profile_name = matches.value_of("profile").unwrap();
        let profile = get_profile(profile_name);

        let version: u32 = matches.value_of("version").unwrap().parse().expect("failed parsing int");
        let id: u64 = matches.value_of("id").unwrap().parse().expect("failed parsing int");

        let password = get_password(profile.pass_secret.to_string());
        let credentials = Credentials{username: profile.username.to_string(), password: password, endpoint: profile.endpoint};
        let page = Page{title: title.to_string(), space: profile.space, source_file: source.to_string(), version: version};
        let content = get_content(page);

        let mut response = update(&credentials, content, id);
        println!("{}", response.text().expect("response text fail"));
        println!("{}", response.status());
    } else if let Some(matches) = matches.subcommand_matches("search") {
        let title = matches.value_of("title").unwrap();

        let profile_name = matches.value_of("profile").unwrap();
        let profile = get_profile(profile_name);

        let password = get_password(profile.pass_secret.to_string());
        let credentials = Credentials{username: profile.username.to_string(), password: password, endpoint: profile.endpoint};

        let page = search(&credentials, profile.space, title.to_string());

        println!("title: {}, id: {}, version: {}", page.title, page.id, page.version.number)
    }
}
