use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

mod config;
mod waka_api;

fn main() {
    env_logger::init();
    let conf = config::init_config();
    // tray menu
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    let enter_api_item = MenuItem::new("Enter WakaTime API key", true, None);

    tray_menu
        .append_items(&[
            &PredefinedMenuItem::about(
                Some("About waka tray"),
                Some(AboutMetadata {
                    name: Some("waka tray".to_string()),
                    copyright: Some("@2024 goverclock".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &enter_api_item,
            &quit_item,
        ])
        .unwrap();

    let event_loop: EventLoop<waka_api::ApiResponse> = EventLoopBuilder::with_user_event().build();

    // a thread to periodically fetch wakatime data, and send it to event loop
    // through the proxy
    let proxy = event_loop.create_proxy();
    waka_api::start_fetch_loop(proxy, conf.api_key.to_owned());

    // main thread to handle all events
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let mut tray_icon = None;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                // We create the icon once the event loop is actually running
                // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_title("startup")
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
            }
            Event::UserEvent(ue) => {
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
            _ => {}
        }

        if let Ok(menu_event) = menu_channel.try_recv() {
            if menu_event.id == quit_item.id() {
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
            } else if menu_event.id == enter_api_item.id() {
                std::process::Command::new("open")
                    .arg(config::config_path())
                    .spawn()
                    .unwrap();
            }
            // info!("{event:?}");
        }

        if let Ok(_tray_event) = tray_channel.try_recv() {
            // info!("{event:?}");
        }
    })
}
