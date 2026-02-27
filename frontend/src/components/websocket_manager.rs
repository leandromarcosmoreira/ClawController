use leptos::*;
use gloo_net::websocket::{Message, WebSocket};
use gloo_net::websocket::events::CloseEvent;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct WebSocketManager {
    pub ws: WebSocket,
}

impl WebSocketManager {
    pub fn new(url: &str) -> Self {
        let ws = WebSocket::open(url).expect("Failed to connect to WebSocket");
        
        Self { ws }
    }

    pub fn send_message(&self, message: WebSocketMessage) {
        if let Ok(json) = serde_json::to_string(&message) {
            self.ws.send(Message::Text(json));
        }
    }

    pub fn on_message<F>(&self, callback: F) 
    where 
        F: Fn(WebSocketMessage) + 'static 
    {
        self.ws.on_message(move |msg| {
            match msg {
                Message::Text(text) => {
                    if let Ok(parsed) = serde_json::from_str::<WebSocketMessage>(&text) {
                        callback(parsed);
                    }
                }
                Message::Bytes(_) => {
                    web_sys::console::log_1(&"Received binary message".into());
                }
            }
        });
    }

    pub fn on_close<F>(&self, callback: F)
    where
        F: Fn(CloseEvent) + 'static
    {
        self.ws.on_close(callback);
    }

    pub fn on_error<F>(&self, callback: F)
    where
        F: Fn(String) + 'static
    {
        self.ws.on_error(move |error| {
            callback(error);
        });
    }
}

#[component]
pub fn WebSocketProvider(url: String) -> impl IntoView {
    let (ws_manager, set_ws_manager) = create_signal(None::<WebSocketManager>);
    let (connection_status, set_connection_status) = create_signal("disconnected".to_string());

    create_effect(move |_| {
        if ws_manager.get().is_none() {
            spawn_local(async move {
                match WebSocket::open(&url) {
                    Ok(ws) => {
                        set_connection_status.set("connected".to_string());
                        set_ws_manager.set(Some(WebSocketManager::new(&url)));
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("WebSocket connection failed: {:?}", e).into());
                        set_connection_status.set("error".to_string());
                    }
                }
            });
        }
    });

    view! {
        <div class="hidden">
            // WebSocket status indicator (hidden but can be used for debugging)
            <span class="websocket-status">
                {connection_status}
            </span>
        </div>
    }
}
