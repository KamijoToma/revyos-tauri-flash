use nusb::MaybeFuture;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::command;
use tauri_plugin_dialog::DialogExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeek;
use tokio::io::AsyncRead;
use anyhow::Context;

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn connect_to_device(device: USBDevice) -> Result<String, String> {
    let device_info: nusb::DeviceInfo = device.try_into()?;
    // Perform connection logic here (stubbed for now)
    Ok(format!(
        "Successfully connected to device: {}",
        device_info.product_string().unwrap_or_else(|| "Unknown")
    ))
}

#[command]
fn flash_uboot_to_ram(file_path: String) -> Result<String, String> {
    // Use the full file path directly
    if std::path::Path::new(&file_path).exists() {
        Ok(format!(
            "Flashed uboot.bin from {} to RAM successfully.",
            file_path
        ))
    } else {
        Err(format!("File not found: {}", file_path))
    }
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
fn flash_files_to_device(
    uboot_path: String,
    boot_path: String,
    root_path: String,
) -> Result<String, String> {
    // Validate all file paths
    if !std::path::Path::new(&uboot_path).exists() {
        return Err(format!("File not found: {}", uboot_path));
    }
    if !std::path::Path::new(&boot_path).exists() {
        return Err(format!("File not found: {}", boot_path));
    }
    if !std::path::Path::new(&root_path).exists() {
        return Err(format!("File not found: {}", root_path));
    }
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

#[command]
fn select_file(window: tauri::Window) -> Result<String, String> {
    let file_path = window
        .dialog()
        .file()
        .blocking_pick_file()
        .ok_or_else(|| "No file selected".to_string())?;
    let path = file_path.into_path().map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct USBDevice {
    vendor_id: u16,
    product_id: u16,
    product_string: String,
    device_address: u8,
}

impl From<nusb::DeviceInfo> for USBDevice {
    fn from(device: nusb::DeviceInfo) -> Self {
        USBDevice {
            vendor_id: device.vendor_id(),
            product_id: device.product_id(),
            product_string: device.product_string().unwrap_or_else(|| "Unknown").to_string(),
            device_address: device.device_address(),
        }
    }
}

// try to convert USBDevice to nusb::DeviceInfo by searching for the device
impl TryFrom<USBDevice> for nusb::DeviceInfo {
    type Error = String;

    fn try_from(device: USBDevice) -> Result<Self, Self::Error> {
        let mut devices = nusb::list_devices().wait().map_err(|e| e.to_string())?;
        devices
            .find(|dev| {
                dev.vendor_id() == device.vendor_id
                    && dev.product_id() == device.product_id
                    && dev.device_address() == device.device_address
            })
            .ok_or_else(|| "Device not found".to_string())
    }
}

#[command]
fn list_usb_devices() -> Result<Vec<USBDevice>, String> {
    let mut devices: Vec<USBDevice> = Vec::new();
    for dev in nusb::list_devices().wait().unwrap() {
        devices.push(dev.into());
    }
    devices.sort_by(|a, b| a.product_string.cmp(&b.product_string));
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test]
    // async fn test_fastboot_protocol() -> anyhow::Result<()> {
    //     let mut devices = fastboot_protocol::nusb::devices()?;
    //     let info = devices
    //         .next()
    //         .ok_or_else(|| anyhow::anyhow!("No Device found"))?;
    //     let mut fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&info)?;

    //     println!("Fastboot version: {}", fb.get_var("version").await?);
    //     Ok(())
    // }

    #[test]
    fn test_list_usb_nusb() {
        for dev in ::nusb::list_devices().wait().unwrap() {
            println!("Device: {:?}", dev);
        }
    }

    #[tokio::test]
    async fn test_fastboot_flash() {
        let devices = nusb::list_devices().wait().unwrap();
        // Get a usb download gadget
        let info = devices.filter(|i| i.vendor_id() == 0x2345).next().unwrap();
        println!("Device: {:#?}", info);
        let mut fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&info).unwrap();
        println!("Fastboot version: {}", fb.get_var("version").await.unwrap());
        // Flash a file
        let file_path = "C:\\Users\\SkyRain\\Downloads\\u-boot-with-spl-lpi4a-16g-main (1).bin";
        let file = tokio::fs::File::open(file_path).await.unwrap();
        let file_size = tokio::fs::metadata(file_path).await.unwrap().len() as u32;
        flash_raw(&mut fb, "ram", file, file_size).await.unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            connect_to_device,
            flash_uboot_to_ram,
            reboot_to_stage2,
            connect_to_stage2,
            flash_files_to_device,
            reboot_device,
            select_file,
            list_usb_devices // Register the new command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn flash_raw<R>(
    fb: &mut fastboot_protocol::nusb::NusbFastBoot,
    target: &str,
    mut file: R,
    file_size: u32,
) -> anyhow::Result<()>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    println!("Uploading raw image directly");
    let mut sender = fb.download(file_size).await?;
    loop {
        let left = sender.left();
        if left == 0 {
            break;
        }
        let buf = sender.get_mut_data(left as usize).await?;
        file.read_exact(buf)
            .await
            .context("Failed to read from file")?;
    }

    sender.finish().await?;
    println!("Flashing data");
    fb.flash(target).await?;

    Ok(())
}