use nusb::MaybeFuture;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct USBDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_string: String,
    pub device_address: u8,
}

impl From<nusb::DeviceInfo> for USBDevice {
    fn from(device: nusb::DeviceInfo) -> Self {
        USBDevice {
            vendor_id: device.vendor_id(),
            product_id: device.product_id(),
            product_string: device.product_string().unwrap_or("Unknown").to_string(),
            device_address: device.device_address(),
        }
    }
}

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

pub fn list_devices() -> Result<Vec<USBDevice>, String> {
    let mut devices: Vec<USBDevice> = Vec::new();
    for dev in nusb::list_devices().wait().unwrap() {
        devices.push(dev.into());
    }
    devices.sort_by(|a, b| a.product_string.cmp(&b.product_string));
    Ok(devices)
}
