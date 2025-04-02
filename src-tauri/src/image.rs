use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// ImageBinaryType represents the type of image binary.
/// It determines how the binary should be flash into the device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageBinaryType {
    UBoot,
    BOOT,
    ROOT,
    SDCARD,
    OTHER(String) // String represents the partition this binary will be flashed to.
}

/// A binary file fetched from web mirror.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBinary {
    pub name: String, // Name of the binary, e.g., "u-boot.bin", "boot.img", etc.
    pub web_path: Option<String>,
    pub local_path: Option<String>,
    pub binary_type: ImageBinaryType,
    pub hash_type: Option<String>, // Hash type for the binary, e.g., SHA256, MD5, etc.
    pub hash_value: Option<String>, // Hash value for the binary, e.g., SHA256, MD5, etc.
}

pub enum ImageBinaryHashError {
    HashTypeNotFound,
    HashValueNotFound,
    HashMismatch,
}

impl ImageBinary {
    pub fn new(name: String, web_path: Option<String>, local_path: Option<String>, binary_type: ImageBinaryType, hash_type: Option<String>, hash_value: Option<String>) -> Result<Self, &'static str> {
        if web_path.is_none() && local_path.is_none() {
            return Err("Either web_path or local_path must be provided.");
        }
        Ok(Self {
            name,
            web_path,
            local_path,
            binary_type,
            hash_type,
            hash_value,
        })
    }

    pub fn try_from_hashmap(map: &HashMap<String, String>, base_url: &String) -> Result<Self, &'static str> {
        let web_path = map.get("address").cloned().ok_or("Web path not found")?;
        let name = map.get("name").cloned().ok_or("Name not found")?;
        // determine the binary type based on the web_path
        let binary_type = if web_path.contains("u-boot") {
            ImageBinaryType::UBoot
        } else if web_path.starts_with("boot") {
            ImageBinaryType::BOOT
        } else if web_path.contains("root") {
            ImageBinaryType::ROOT
        }else if web_path.contains("sdcard"){
            ImageBinaryType::SDCARD
        } else {
            ImageBinaryType::OTHER(web_path.clone())
        };
        Self::new(name, Some(format!("{}/{}", base_url, web_path)), None, binary_type, None, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariant {
    pub name: String,
    pub image_binarys: Vec<ImageBinary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVersion {
    pub version: String,
    pub image_variants: Vec<ImageVariant>,
}
