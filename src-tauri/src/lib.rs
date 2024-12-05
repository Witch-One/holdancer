use formation::Formation;

mod formation;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_formation(ts: i32) -> String {
    let res = formation::get_current_formation_by_timestamp(ts);
    serde_json::to_string(&res).expect("Error serializing formation")
}

#[tauri::command]
fn update_formation(ts: i32, formation: String) -> String {
    let f: formation::Formation = serde_json::from_str(&formation).unwrap();
    formation::update_formation(ts, f).unwrap();
    "Formation updated".to_string()
}

#[tauri::command]
fn add_dancer(formation_id: i32) -> String {
    let res = formation::add_new_dancer(formation_id);
    match res {
        Ok(formation) => serde_json::to_string(&formation).expect("Error serializing formation"),
        Err(e) => e.to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_formation,
            update_formation,
            add_dancer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
