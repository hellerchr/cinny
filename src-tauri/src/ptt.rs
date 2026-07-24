use rdev::{listen, EventType, Key};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

static PTT_KEY: Mutex<Option<Key>> = Mutex::new(None);

#[tauri::command]
pub fn set_ptt_key(key: Option<String>) {
    *PTT_KEY.lock().unwrap() = key.as_deref().and_then(parse_key);
}

fn parse_key(code: &str) -> Option<Key> {
    match code {
        "MetaLeft" => Some(Key::MetaLeft),
        "MetaRight" => Some(Key::MetaRight),
        "ShiftLeft" => Some(Key::ShiftLeft),
        "ShiftRight" => Some(Key::ShiftRight),
        "ControlLeft" => Some(Key::ControlLeft),
        "ControlRight" => Some(Key::ControlRight),
        "AltLeft" => Some(Key::Alt),
        "AltRight" => Some(Key::AltGr),
        _ => None,
    }
}

pub fn start_listener(app: AppHandle) {
    #[cfg(target_os = "macos")]
    unsafe {
        CGRequestListenEventAccess();
    }

    std::thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            let watched = *PTT_KEY.lock().unwrap();
            if let Some(key) = watched {
                match event.event_type {
                    EventType::KeyPress(k) if k == key => {
                        let _ = app.emit("ptt-key-press", ());
                    }
                    EventType::KeyRelease(k) if k == key => {
                        let _ = app.emit("ptt-key-release", ());
                    }
                    _ => {}
                }
            }
        }) {
            eprintln!("ptt listener error: {error:?}");
        }
    });
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGRequestListenEventAccess() -> bool;
}
