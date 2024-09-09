use core::time;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;
use std::thread;
use tao::event_loop::EventLoopProxy;

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub data: Vec<DataItem>,
    end: String,
    start: String,
    timezone: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct DataItem {
    color: Option<String>,
    pub duration: f64,
    project: String,
    time: f64,
}

pub fn start_fetch_loop(proxy: EventLoopProxy<ApiResponse>, api_key: String) {
    thread::spawn(move || loop {
        match fetch_wakatime(&api_key) {
            Ok(response) => {
                proxy.send_event(response).unwrap();
            }
            Err(e) => {
                println!("fail to fetch wakatime api, err={e}");
            }
        }
        thread::sleep(time::Duration::from_secs(60));
    });
}

fn fetch_wakatime(api_key: &str) -> Result<ApiResponse, Box<dyn Error>> {
    println!("fetching");
    let client = Client::new();
    let url = format!(
        "https://wakatime.com/api/v1/users/current/durations?date={}",
        chrono::Local::now().format("%Y-%m-%d")
    );

    let response = client.get(url).basic_auth(api_key, Some("")).send()?;
    if response.status().is_success() {
        let api_response: ApiResponse = response.json()?;
        Ok(api_response)
    } else {
        Err(Box::from(response.status().as_str()))
    }
}
