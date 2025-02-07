use once_cell::sync::OnceCell;
use std::sync::mpsc::Sender;

pub static GLOBAL_EVENT_SENDER: OnceCell<Sender<EventMessage>> = OnceCell::new();

pub enum EventMessage {
    Emit(String, String),
}

pub fn global_emit(name: &str, payload: &str) {
    if let Some(sender) = GLOBAL_EVENT_SENDER.get() {
        if let Err(e) = sender.send(EventMessage::Emit(name.to_string(), payload.to_string())) {
            if cfg!(dev) {
                println!("{:?}", e);
            }
        }
    } else {
        if cfg!(dev) {
            println!("Global event sender is not initialized");
        }
    }
}
