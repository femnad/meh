extern crate serde;
extern crate attohttpc;

use serde::Deserialize;
use serde_json::value::Value;

pub struct Credentials {
    pub username: String,
    pub password: String,
    pub endpoint: String,
}

#[derive(Clone, Deserialize)]
pub struct Version {
    pub number: u32,
}

#[derive(Clone, Deserialize)]
pub struct Space {
    pub key: String
}

#[derive(Clone, Deserialize)]
pub struct ConfluencePage {
    pub id: String,
    pub title: String,
    pub version: Version,
    pub space: Space,
}

#[derive(Deserialize)]
pub struct View {
    pub value: String
}

#[derive(Deserialize)]
pub struct Body {
    pub view: View
}

#[derive(Deserialize)]
pub struct ContentView {
    pub id: String,
    pub title: String,
    pub body: Body
}

#[derive(Deserialize)]
struct SearchResults {
    results: Vec<ConfluencePage>
}

pub fn get_endpoint(credentials: &Credentials) -> String {
    format!("{}/{}", credentials.endpoint, "confluence/rest/api/content")
}

pub fn update(credentials: &Credentials, content: Value, id: String) -> Result<(), String> {
    let endpoint = format!("{}/{}", get_endpoint(credentials), id);
    let response = attohttpc::put(endpoint)
        .basic_auth(&credentials.username, Option::Some(credentials.password.clone()))
        .header("Content-Type", "application/json")
        .json(&content)
        .expect("Error building put request")
        .send()
        .expect("Error sending put request");

    if !response.is_success() {
        return Err(response.text().expect("error getting response text"))
    }

    return Ok(())
}

pub fn create(credentials: &Credentials, content: Value) -> Result<(), String> {
    let endpoint = format!("{}", get_endpoint(credentials));
    let response = attohttpc::post(endpoint)
        .basic_auth(&credentials.username, Some(&credentials.password))
        .json(&content)
        .expect("Error building post request")
        .send()
        .expect("Error sending post request");
    if response.is_success() {
        return Ok(())
    }
    Err(response.text().expect("Error getting response"))
}

pub fn search(credentials: &Credentials, space: String, title: String) -> Result<ConfluencePage, String> {
    let endpoint = format!("{endpoint}?spaceKey={space}&title={title}&expand=version,space",
                           endpoint=get_endpoint(credentials), space=space, title=title);
    let response = attohttpc::get(endpoint)
        .basic_auth(&credentials.username, Some(&credentials.password))
        .send()
        .expect("Error sending get request");
    if !response.is_success() {
        return Err(response.text().expect("Error getting response text"))
    }
    let search_results: SearchResults = response.json()
        .expect("cannot extract JSON");
    if search_results.results.len() == 0 {
        return Err(format!("No page found with title {}", title))
    }
    let first_result = &search_results.results[0];
    Ok(first_result.clone())
}

pub fn get(credentials: &Credentials, id: String) -> Result<ContentView, String> {
    let endpoint = format!("{endpoint}/{id}?expand=body.view", endpoint=get_endpoint(credentials), id=id);
    let response = attohttpc::get(endpoint.as_str())
        .basic_auth(&credentials.username, Some(&credentials.password))
        .send()
        .expect("get by id fail");
    if !response.is_success() {
        return Err(response.text().expect("error getting response text"))
    }
    let content_view: ContentView = response.json().expect("error getting response as json");
    return Ok(content_view)
}
