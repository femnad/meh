#[macro_use]
extern crate clap;
extern crate dirs;
extern crate serde;

use std::process::Command;
use std::io::prelude::*;
use std::fs::File;

use clap::{App, Arg, SubCommand, AppSettings};
use serde::Deserialize;

mod confluence;
mod ops;

const DEFAULT_PROFILE: &str = "default";
const PROFILE_FILE_SUFFIX: &str = "meh/meh.yaml";


#[derive(Deserialize)]
struct Profile {
    name: String,
    username: String,
    pass_secret: String,
    endpoint: String,
    space: String,
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


fn get_password(secret_name: String) -> String {
    let output = Command::new("pass")
        .arg(secret_name)
        .output()
        .expect("fail pass");
    let lines = String::from_utf8(output.stdout).expect("failage");
    let v: Vec<&str> = lines.trim().split('\n').collect();
    v[0].to_string()
}


fn main() {
    let matches = App::new("meh")
        .version(crate_version!())
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
        .subcommand(SubCommand::with_name("get")
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
                .help("title of the page to get")
                .takes_value(true)
                .required(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let title = matches.value_of("title").unwrap();
        let source = matches.value_of("source").unwrap();

        let profile_name = matches.value_of("profile").unwrap();
        let profile = get_profile(profile_name);

        let password = get_password(profile.pass_secret.to_string());
        let credentials = confluence::Credentials{username: profile.username.to_string(), password: password, endpoint: profile.endpoint};

        let result = ops::create(&credentials, title.to_string(), profile.space, source.to_string());

        match result {
            Ok(()) => println!("create success"),
            Err(err) => println!("create fail {}", err),
        };
    } else if let Some(matches) = matches.subcommand_matches("update") {
        let title = matches.value_of("title").unwrap();
        let source = matches.value_of("source").unwrap();

        let profile_name = matches.value_of("profile").unwrap();
        let profile = get_profile(profile_name);
        let password = get_password(profile.pass_secret.to_string());
        let credentials = confluence::Credentials{username: profile.username.to_string(), password: password, endpoint: profile.endpoint};

        let result = ops::update(&credentials, profile.space, title.to_string(),  source.to_string());
        match result {
            Ok(()) => println!("update success"),
            Err(text) => println!("update fail {}", text),
        }
    } else if let Some(matches) = matches.subcommand_matches("search") {
        let title = matches.value_of("title").unwrap();

        let profile_name = matches.value_of("profile").unwrap();
        let profile = get_profile(profile_name);

        let password = get_password(profile.pass_secret.to_string());
        let credentials = confluence::Credentials{username: profile.username.to_string(), password: password, endpoint: profile.endpoint};

        let response = confluence::search(&credentials, profile.space, title.to_string());
        match response {
            Ok(page) => println!("title: {}, id: {}, version: {}", page.title, page.id,
                                 page.version.number),
            Err(text) => println!("search fail {}", text),
        }

    } else if let Some(matches) = matches.subcommand_matches("get") {
        let title = matches.value_of("title").unwrap();

        let profile_name = matches.value_of("profile").unwrap();
        let profile = get_profile(profile_name);

        let password = get_password(profile.pass_secret.to_string());
        let credentials = confluence::Credentials{username: profile.username.to_string(), password: password, endpoint: profile.endpoint};

        let lookup = ops::get(&credentials, title.to_string(), profile.space);
        match lookup {
            Ok(content_view) => println!("{}", content_view.body.view.value),
            Err(text) => println!("get fail {}", text),
        }
    }
}
