use core::time;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{error::Error, thread};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

// TODO: save API key somewhere else, ask user to fill it on first launch
const API_KEY: &str = "";

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct ApiResponse {
    data: Vec<DataItem>,
    end: String,
    start: String,
    timezone: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct DataItem {
    color: Option<String>,
    duration: f64,
    project: String,
    time: f64,
}

fn fetch_wakatime() -> Result<ApiResponse, Box<dyn Error>> {
    println!("fetching");
    let client = Client::new();
    let url = format!(
        "https://wakatime.com/api/v1/users/current/durations?date={}",
        chrono::Utc::now().format("%Y-%m-%d")
    );

    let response = client.get(url).basic_auth(API_KEY, Some("")).send()?;
    if response.status().is_success() {
        let api_response: ApiResponse = response.json()?;
        Ok(api_response)
    } else {
        Err(Box::from(response.status().as_str()))
    }
}

fn main() {
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);

    tray_menu
        .append_items(&[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("waka tray".to_string()),
                    copyright: Some("@2024 goverclock".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .unwrap();

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let mut tray_icon = None;
    // let event_loop = EventLoopBuilder::new().build();
    let event_loop: EventLoop<ApiResponse> = EventLoopBuilder::with_user_event().build();

    // a thread to periodically fetch wakatime data, and send it to event loop
    // through the proxy
    let proxy = event_loop.create_proxy();
    thread::spawn(move || loop {
        match fetch_wakatime() {
            Ok(response) => {
                proxy.send_event(response).unwrap();
            }
            Err(e) => {
                println!("fail to fetch wakatime api, err={e}");
            }
        }
        thread::sleep(time::Duration::from_secs(60));
    });

    let mut cnt = 0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        cnt += 1;
        if cnt % 100 == 0 {
            println!("{}", cnt);
        }
        // println!("{:#?}", event);

        if let Event::NewEvents(tao::event::StartCause::Init) = event {
            // We create the icon once the event loop is actually running
            // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
            tray_icon = Some(
                TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu.clone()))
                    .with_tooltip("waka tray - coding time at tray")
                    .with_title("123")
                    .build()
                    .unwrap(),
            );

            // We have to request a redraw here to have the icon actually show up.
            // Tao only exposes a redraw method on the Window so we use core-foundation directly.
            #[cfg(target_os = "macos")]
            unsafe {
                use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};

                let rl = CFRunLoopGetMain();
                CFRunLoopWakeUp(rl);
            }
        } else if let Event::UserEvent(ue) = event {
            // received some coding stat from the fetching thread
            let mut seconds = 0f64;
            for d in ue.data {
                seconds += d.duration;
            }
            let seconds = seconds as u32;
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            tray_icon
                .as_ref()
                .unwrap()
                .set_title(Some(format!("{hours}h{minutes}m")));
        }

        if let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == quit_item.id() {
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
            }
            // println!("{event:?}");
        }

        if let Ok(_tray_event) = tray_channel.try_recv() {
            // println!("{event:?}");
        }
    })
}
