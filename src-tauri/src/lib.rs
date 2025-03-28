// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::command;

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn connect_to_device() -> Result<String, String> {
    // Stub implementation
    Ok("Connected to stage 1 USB device.".to_string())
}

#[command]
fn flash_uboot_to_ram(file_path: String) -> Result<String, String> {
    // Stub implementation
    Ok(format!("Flashed uboot.bin from {} to RAM successfully.", file_path))
}

#[command]
fn reboot_to_stage2() -> Result<String, String> {
    // Stub implementation
    Ok("Rebooted to stage 2.".to_string())
}

#[command]
fn connect_to_stage2() -> Result<String, String> {
    // Stub implementation
    Ok("Connected to stage 2 USB device.".to_string())
}

#[command]
fn flash_files_to_device(uboot_path: String, boot_path: String, root_path: String) -> Result<String, String> {
    // Stub implementation
    Ok(format!(
        "Flashed uboot: {}, boot: {}, root: {} successfully.",
        uboot_path, boot_path, root_path
    ))
}

#[command]
fn reboot_device() -> Result<String, String> {
    // Stub implementation
    Ok("Device rebooted successfully.".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            connect_to_device,
            flash_uboot_to_ram,
            reboot_to_stage2,
            connect_to_stage2,
            flash_files_to_device,
            reboot_device
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
