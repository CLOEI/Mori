use gt_core::Bot;
use gtitem_r::structs::ItemDatabase;
use serde_json::Value;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use tao::event_loop::EventLoopBuilder;
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::platform::windows::EventLoopBuilderExtWindows;
use tao::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use wry::WebViewBuilder;

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    TokenReceived,
}

// THIS IS JUST A SCRATCH, NOT A FINAL IMPLEMENTATION, DO NOT USE THIS AS IS

fn main() {
    let token_fetch = |url: String| {
        let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build();
        let proxy = event_loop.create_proxy();
        let token_mutex = Arc::new(Mutex::new("".to_string()));

        let window = WindowBuilder::new()
            .with_title("Login Required")
            .build(&event_loop)
            .unwrap();

        let initialization_script = r#"
                window.addEventListener('DOMContentLoaded', (event) => {
                    if (window.location.href.includes("validate")) {
                        const content = document.body.innerHTML;
                        window.ipc.postMessage(content);
                    }
                });
            "#;

        let token_mutex_clone = Arc::clone(&token_mutex);
        let _webview = WebViewBuilder::new()
            .with_url(&url)
            .with_initialization_script(initialization_script)
            .with_ipc_handler(move |req| {
                let data = req.body();
                println!("IPC Data received: {}", data);

                if data.contains("token") {
                    match serde_json::from_str::<Value>(data) {
                        Ok(json) => {
                            if let Some(token_value) = json.get("token") {
                                if let Some(token) = token_value.as_str() {
                                    let mut token_lock = token_mutex_clone.lock().unwrap();
                                    *token_lock = token.to_string();
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse JSON: {}", e);
                        }
                    }
                }
                proxy.send_event(UserEvent::TokenReceived).unwrap();
            })
            .build(&window)
            .unwrap();

        println!(
            "WebView thread: WebView is running. Please log in and navigate to the validation page..."
        );

        event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::MainEventsCleared => {}
                Event::UserEvent(UserEvent::TokenReceived) => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        });

        let token_lock = token_mutex.lock().unwrap();
        let token = token_lock.clone();
        println!("Extracted token: {}", token);
        token
    };

    let mut bots: Vec<Arc<Bot>> = vec![];

    let item_database = Arc::new(RwLock::new(ItemDatabase::new()));

    let bot = Arc::new(Bot::new(
        vec!["".to_string(), "".to_string()],
        Some(Box::new(token_fetch)),
        item_database,
    ));
    let bot_clone = Arc::clone(&bot);
    let t = thread::spawn(move || {
        bot_clone.logon(None);
    });
    bots.push(bot);

    t.join().unwrap();
}
