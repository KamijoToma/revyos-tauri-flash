use serde::Serialize;
use tauri::{command, ipc::Channel};
use tauri_plugin_dialog::DialogExt;
use crate::usb::{USBDevice, list_devices};
use crate::flash::flash;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum UploadProgressEvent {
    #[serde(rename_all = "camelCase")]
    Progress { current: u64, total: u64 },
}

#[command]
pub fn connect_to_device(device: USBDevice) -> Result<String, String> {
    let device_info: nusb::DeviceInfo = device.try_into()?;
    // Perform connection logic here (stubbed for now)
    Ok(format!(
        "Successfully connected to device: {}",
        device_info.product_string().unwrap_or("Unknown")
    ))
}

#[command]
pub async fn reboot_device(device: USBDevice) -> Result<String, String> {
    // Stub implementation
    let device_info: nusb::DeviceInfo = device.try_into()?;
    let mut fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&device_info).unwrap();
    println!("Fastboot version: {}", fb.get_var("version").await.unwrap());
    fb.reboot().await.map_err(|e| e.to_string())?;
    Ok("Rebooted device.".to_string())
}

#[command]
pub async fn flash_to_partition(
    file_path: String,
    partition: String,
    device: USBDevice,
    on_event: Channel<UploadProgressEvent>,
) -> Result<String, String> {
    // Validate file path
    if (!std::path::Path::new(&file_path).exists()) {
        return Err(format!("File not found: {}", file_path));
    }
    let device_info: nusb::DeviceInfo = device.try_into()?;
    let mut fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&device_info)
        .map_err(|e| e.to_string())?;
    println!(
        "Fastboot version: {}",
        fb.get_var("version").await.map_err(|e| e.to_string())?
    );
    flash(&mut fb, &partition, std::path::Path::new(&file_path), move |c, t| on_event.send(UploadProgressEvent::Progress { current: c, total: t }).unwrap()).await.map_err(|e| e.to_string())?;
    Ok(format!(
        "Flashed file to partition {} on device: {}",
        partition,
        device_info.product_string().unwrap_or("Unknown")
    )
    .to_string())
}

#[command]
pub fn select_file(window: tauri::Window) -> Result<String, String> {
    let file_path = window
        .dialog()
        .file()
        .blocking_pick_file()
        .ok_or_else(|| "No file selected".to_string())?;
    let path = file_path.into_path().map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[command]
pub fn list_usb_devices() -> Result<Vec<USBDevice>, String> {
    list_devices()
}

// 增加一个新的命令，用于获取LPi4A镜像版本列表
#[tauri::command]
pub async fn fetch_lpi4a_image_versions() -> Result<Vec<crate::image::ImageVersion>, String> {
    crate::html_parser::fetch_and_parse_lpi4a_image_all(None)
        .await
        .map_err(|e| e.to_string())
}