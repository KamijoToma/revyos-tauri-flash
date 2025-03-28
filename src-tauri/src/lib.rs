use std::io::SeekFrom;

use anyhow::bail;
use nusb::MaybeFuture;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::command;
use tauri::ipc::Channel;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeek;
use tokio::io::AsyncRead;
use tokio::io::AsyncSeekExt;
use anyhow::Context;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
enum UploadProgressEvent{
    #[serde(rename_all = "camelCase")]
    Progress {
        current: u64,
        total: u64,
    }
}

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
async fn flash_uboot_to_ram(file_path: String, device: USBDevice, on_event: Channel<UploadProgressEvent>) -> Result<String, String> {
    // Use the full file path directly
    if !std::path::Path::new(&file_path).exists() {
        return Err(format!("File not found: {}", file_path));
    }
    let device_info: nusb::DeviceInfo = device.try_into()?;
    let mut fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&device_info)
        .map_err(|e| e.to_string())?;
    println!(
        "Fastboot version: {}",
        fb.get_var("version").await.map_err(|e| e.to_string())?
    );
    flash(&mut fb, "ram", std::path::Path::new(&file_path), move |c, t| on_event.send(UploadProgressEvent::Progress { current: c, total: t }).unwrap()).await.map_err(|e| e.to_string())?;
    Ok(format!(
        "Flashed u-boot to RAM on device: {}",
        device_info.product_string().unwrap_or_else(|| "Unknown")
    )
    .to_string())
}

#[command]
async fn reboot_device(device: USBDevice) -> Result<String, String> {
    // Stub implementation
    let device_info: nusb::DeviceInfo = device.try_into()?;
    let mut fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&device_info).unwrap();
    println!("Fastboot version: {}", fb.get_var("version").await.unwrap());
    fb.reboot().await.map_err(|e| e.to_string())?;
    Ok("Rebooted device.".to_string())
}

#[command]
async fn flash_files_to_device(
    uboot_path: String,
    boot_path: String,
    root_path: String,
    device: USBDevice,
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
    let flash_map = [
        ("uboot", uboot_path),
        ("boot", boot_path),
        ("root", root_path),
    ];
    let device_info: nusb::DeviceInfo = device.try_into()?;
    let mut fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&device_info)
        .map_err(|e| e.to_string())?;
    println!(
        "Fastboot version: {}",
        fb.get_var("version").await.map_err(|e| e.to_string())?
    );
    for (target, path) in flash_map.iter() {
        println!("Flashing {} to device", target);
        flash(&mut fb, target, std::path::Path::new(path), |_, _| {}).await.map_err(|e| e.to_string())?;
    }
    Ok("Flashed files successfully.".to_string())
}

#[command]
async fn flash_to_partition(
    file_path: String,
    partition: String,
    device: USBDevice,
    on_event: Channel<UploadProgressEvent>,
) -> Result<String, String> {
    // Validate file path
    if !std::path::Path::new(&file_path).exists() {
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
        device_info.product_string().unwrap_or_else(|| "Unknown")
    )
    .to_string())
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
        flash(&mut fb, "ram", std::path::Path::new(file_path), |_, _| {}).await.unwrap();
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
            reboot_device,
            flash_to_partition,
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

use android_sparse_image::{
    split::split_image, ChunkHeader, FileHeader, FileHeaderBytes, CHUNK_HEADER_BYTES_LEN,
};

async fn flash<F>(
    fb: &mut fastboot_protocol::nusb::NusbFastBoot,
    target: &str,
    file: &std::path::Path,
    mut progress_callback: F,
) -> anyhow::Result<()>
where
    F: FnMut(u64, u64) + Send + 'static,
{
    let max_download = fb.get_var("max-download-size").await?;
    let max_download = fastboot_protocol::protocol::parse_u32_hex(&max_download)
        .with_context(|| anyhow::anyhow!("Failed to parse max download size: {max_download}"))?;
    println!("Max download size: {max_download}");

    let mut f = tokio::fs::File::open(file).await?;
    let mut header_bytes = FileHeaderBytes::default();
    f.read_exact(&mut header_bytes).await?;
    let splits = match FileHeader::from_bytes(&header_bytes) {
        Ok(header) => {
            println!("Preparing to flash android sparse image");
            let mut chunks = vec![];
            for _ in 0..header.chunks {
                let mut chunk_bytes = [0; CHUNK_HEADER_BYTES_LEN];
                f.read_exact(&mut chunk_bytes).await?;
                let chunk = ChunkHeader::from_bytes(&chunk_bytes)?;

                f.seek(SeekFrom::Current(chunk.data_size() as i64)).await?;
                chunks.push(chunk);
            }
            split_image(&header, &chunks, max_download)?
        }
        Err(android_sparse_image::ParseError::UnknownMagic) => {
            f.seek(SeekFrom::Start(0))
                .await
                .context("Seeking back to the start")?;
            let file_size = f
                .seek(SeekFrom::End(0))
                .await
                .context("Seek for determining file size")?;
            if file_size < max_download as u64 {
                f.seek(SeekFrom::Start(0))
                    .await
                    .context("Seeking back to the start")?;
                return flash_raw(fb, target, f, file_size as u32).await;
            }
            android_sparse_image::split::split_raw(file_size as usize, max_download)?
        }
        Err(e) => bail!("Failed to parse sparse image: {e}"),
    };

    println!("Flashing in {} parts", splits.len());
    let total_parts = splits.len() as u64;
    for (i, split) in splits.iter().enumerate() {
        println!("Downloading part {i}");
        let mut sender = fb.download(split.sparse_size() as u32).await?;

        sender.extend_from_slice(&split.header.to_bytes()).await?;
        for chunk in &split.chunks {
            sender.extend_from_slice(&chunk.header.to_bytes()).await?;
            f.seek(SeekFrom::Start(chunk.offset as u64))
                .await
                .context("Failed to seek input file")?;
            let mut left = chunk.size;
            while left > 0 {
                let buf = sender.get_mut_data(left).await?;
                left -= f
                    .read_exact(buf)
                    .await
                    .context("Failed to read from file")?;
            }
        }
        sender.finish().await?;
        println!("Flashing Part {i}");
        fb.flash(target).await?;
        progress_callback(i as u64 + 1, total_parts); // Update progress
    }

    Ok(())
}