use std::fs::File;

use std::io::prelude::*;
use crate::confluence;
use serde_json::json;
use serde_json::value::Value;

struct Content {
    title: String,
    space: String,
    source_file: String,
    version: u32,
}

fn get_content(content: Content) -> Value {
    let mut file = File::open(content.source_file).expect("unable to open file");
    let mut page_body = String::new();
    file.read_to_string(&mut page_body).expect("unable to read file");
    let content = json!({
        "version": {
            "number": content.version
        },
        "type": "page",
        "space": {
            "key": content.space
        },
        "title": content.title,
        "body": {
            "storage": {
                "value": page_body,
                "representation": "wiki"
            }
        }
    });
    content
}

fn get_update_content(current_page: confluence::ConfluencePage, source_file: String) -> Value {
    let content = Content{title: current_page.title, space: current_page.space.key, source_file: source_file, version: current_page.version.number + 1};
    get_content(content)
}

pub fn update(credentials: &confluence::Credentials, space: String, title: String, source_file: String) -> Result<(), String> {
    let result = confluence::search(credentials, space, title);
    match result {
        Ok(page) => {
            let update_content = get_update_content(page.clone(), source_file);
            confluence::update(credentials, update_content, page.id)
        },
        Err(reason) => {
            return Err(reason)
        }
    }
}

pub fn create(credentials: &confluence::Credentials, title: String, space: String, source_file: String) -> Result<(), String> {
    let page = Content{title: title, space: space, source_file: source_file, version: 1};
    let content = get_content(page);
    confluence::create(&credentials, content)
}

