// Prevents additional console window on Windows in debug mode.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod desktop_lib;

#[tauri::command]
fn server_version(server: String) -> Result<String, String> {
    desktop_lib::server_version(&server, 5000).ok_or_else(|| "Could not connect to server".to_owned())
}

#[tauri::command]
fn task_list(server: String) -> Result<serde_json::Value, String> {
    let tasks = desktop_lib::task_list(&server, 5000);
    serde_json::to_value(tasks).map_err(|e| e.to_string())
}

#[tauri::command]
fn task_get(server: String, task_id: String) -> Result<serde_json::Value, String> {
    desktop_lib::task_get(&server, &task_id, 5000)
        .ok_or_else(|| format!("Task '{}' not found", task_id))
}

#[tauri::command]
fn task_create(server: String, id: String, source: String, destination: String) -> Result<serde_json::Value, String> {
    desktop_lib::task_create(&server, &id, &source, &destination, 30000)
        .ok_or_else(|| "Task creation failed".to_owned())
}

#[tauri::command]
fn task_queue(server: String, task_id: String) -> Result<bool, String> {
    Ok(desktop_lib::task_queue(&server, &task_id, 5000))
}

#[tauri::command]
fn task_start(server: String) -> Result<String, String> {
    desktop_lib::task_start(&server, 5000).ok_or_else(|| "No queued task".to_owned())
}

#[tauri::command]
fn task_pause(server: String, task_id: String) -> Result<bool, String> {
    Ok(desktop_lib::task_pause(&server, &task_id, 5000))
}

#[tauri::command]
fn task_resume(server: String, task_id: String) -> Result<bool, String> {
    Ok(desktop_lib::task_resume(&server, &task_id, 5000))
}

#[tauri::command]
fn task_remove(server: String, task_id: String) -> Result<bool, String> {
    Ok(desktop_lib::task_remove(&server, &task_id, 5000))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            server_version,
            task_list,
            task_get,
            task_create,
            task_queue,
            task_start,
            task_pause,
            task_resume,
            task_remove,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nexum desktop");
}
