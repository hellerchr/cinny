#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ptt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![ptt::set_ptt_key])
        .setup(|app| {
            ptt::start_listener(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
