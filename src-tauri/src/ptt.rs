use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModifierKey {
    MetaLeft,
    MetaRight,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
}

static PTT_KEY: Mutex<Option<ModifierKey>> = Mutex::new(None);

#[tauri::command]
pub fn set_ptt_key(key: Option<String>) {
    *PTT_KEY.lock().unwrap() = key.as_deref().and_then(parse_key);
}

fn parse_key(code: &str) -> Option<ModifierKey> {
    match code {
        "MetaLeft" => Some(ModifierKey::MetaLeft),
        "MetaRight" => Some(ModifierKey::MetaRight),
        "ShiftLeft" => Some(ModifierKey::ShiftLeft),
        "ShiftRight" => Some(ModifierKey::ShiftRight),
        "ControlLeft" => Some(ModifierKey::ControlLeft),
        "ControlRight" => Some(ModifierKey::ControlRight),
        "AltLeft" => Some(ModifierKey::AltLeft),
        "AltRight" => Some(ModifierKey::AltRight),
        _ => None,
    }
}

fn emit_key_event(app: &AppHandle, pressed: bool) {
    let event = if pressed { "ptt-key-press" } else { "ptt-key-release" };
    let _ = app.emit(event, ());
}

#[cfg(target_os = "macos")]
pub fn start_listener(app: AppHandle) {
    use core_foundation::runloop::CFRunLoop;
    use core_graphics::event::{
        CallbackResult, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
        CGEventTapPlacement, CGEventType, EventField,
    };

    // (virtual key code, modifier flag) for left/right modifier keys
    fn key_info(key: ModifierKey) -> (i64, CGEventFlags) {
        match key {
            ModifierKey::MetaLeft => (0x37, CGEventFlags::CGEventFlagCommand),
            ModifierKey::MetaRight => (0x36, CGEventFlags::CGEventFlagCommand),
            ModifierKey::ShiftLeft => (0x38, CGEventFlags::CGEventFlagShift),
            ModifierKey::ShiftRight => (0x3C, CGEventFlags::CGEventFlagShift),
            ModifierKey::ControlLeft => (0x3B, CGEventFlags::CGEventFlagControl),
            ModifierKey::ControlRight => (0x3E, CGEventFlags::CGEventFlagControl),
            ModifierKey::AltLeft => (0x3A, CGEventFlags::CGEventFlagAlternate),
            ModifierKey::AltRight => (0x3D, CGEventFlags::CGEventFlagAlternate),
        }
    }

    unsafe {
        CGRequestListenEventAccess();
    }

    std::thread::spawn(move || {
        let result = CGEventTap::with_enabled(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::FlagsChanged],
            move |_proxy, _event_type, event| {
                let watched = *PTT_KEY.lock().unwrap();
                if let Some((keycode, flag)) = watched.map(key_info) {
                    let pressed_keycode =
                        event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                    if pressed_keycode == keycode {
                        emit_key_event(&app, event.get_flags().contains(flag));
                    }
                }
                CallbackResult::Keep
            },
            CFRunLoop::run_current,
        );
        if result.is_err() {
            eprintln!("ptt listener error: failed to install event tap");
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub fn start_listener(app: AppHandle) {
    use rdev::{listen, EventType, Key};

    fn to_rdev_key(key: ModifierKey) -> Key {
        match key {
            ModifierKey::MetaLeft => Key::MetaLeft,
            ModifierKey::MetaRight => Key::MetaRight,
            ModifierKey::ShiftLeft => Key::ShiftLeft,
            ModifierKey::ShiftRight => Key::ShiftRight,
            ModifierKey::ControlLeft => Key::ControlLeft,
            ModifierKey::ControlRight => Key::ControlRight,
            ModifierKey::AltLeft => Key::Alt,
            ModifierKey::AltRight => Key::AltGr,
        }
    }

    std::thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            let watched = (*PTT_KEY.lock().unwrap()).map(to_rdev_key);
            if let Some(key) = watched {
                match event.event_type {
                    EventType::KeyPress(k) if k == key => emit_key_event(&app, true),
                    EventType::KeyRelease(k) if k == key => emit_key_event(&app, false),
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
