mod usb;
mod flash;
mod commands;
mod html_parser;
mod image;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::connect_to_device,
            commands::reboot_device,
            commands::flash_to_partition,
            commands::select_file,
            commands::list_usb_devices
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
