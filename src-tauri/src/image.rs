use std::{collections::HashMap, path::PathBuf};
use serde::{Serialize, Deserialize};
use futures_lite::stream::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use async_compression::tokio::bufread::ZstdDecoder;
/// 表示进度类型的枚举，用于区分下载还是解压缩过程;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressType {
    Download,
    Extract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// ImageBinaryType represents the type of image binary.
pub enum ImageBinaryType {
    UBoot,
    Boot,
    Root,
    Sdcard,
    Other(String) // String represents the partition this binary will be flashed to.
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

#[allow(dead_code)]
pub enum ImageBinaryHashError {
    HashTypeNotFound,// Hash type for the binary, e.g., SHA256, MD5, etc.
    HashValueNotFound,// Hash value for the binary, e.g., SHA256, MD5, etc.
    Mismatch,
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
            ImageBinaryType::Boot
        } else if web_path.contains("root") {
            ImageBinaryType::Root
        }else if web_path.contains("sdcard"){
            ImageBinaryType::Sdcard
        } else {
            ImageBinaryType::Other(web_path.clone())
        };
        Self::new(name, Some(format!("{}/{}", base_url, web_path)), None, binary_type, None, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariant {
    pub name: String,
    pub image_binarys: Vec<ImageBinary>,
}

impl ImageVariant {

    fn get_local_path(name: &String, binary_name: &String) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("revyos-imager");
        path.push(name);
        path.push(if binary_name.ends_with(".zst") {
            binary_name.trim_end_matches(".zst").to_string()
        } else {
            binary_name.to_string()
        });
        path
    }

    async fn check_zst_file(file_path: &PathBuf) -> Result<bool, String> {
        let file = tokio::fs::File::open(file_path).await.map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer).await.map_err(|e| e.to_string())?;
        Ok(buffer == [0x28, 0xb5, 0x2f, 0xfd])
    }

    /// Download all binaries in this variant.
    pub async fn download_binaries<F>(&mut self, mut progress_callback: F) -> Result<(), String>
    where
        F: FnMut(&str, u64, u64, ProgressType),
    {
        let client = reqwest::Client::new();
        for binary in &mut self.image_binarys {
            if let Some(local_path) = &binary.local_path {
                println!("Using local binary: {}", local_path);
                continue;
            }
            if let Some(web_path) = &binary.web_path {
                let file_path = ImageVariant::get_local_path(&self.name, &binary.name);
                // first we download the file to a temp file in the same dir
                // then we rename it to the final name
                let temp_file_path = file_path.with_extension("tmp");
                std::fs::create_dir_all(file_path.parent().unwrap()).map_err(|e| e.to_string())?;
                println!("Downloading {} from {}", temp_file_path.to_string_lossy(), web_path);
                let response = client.get(web_path).send().await.map_err(|e| e.to_string())?;
                let mut file = tokio::fs::File::create(&temp_file_path).await.map_err(|e| e.to_string())?;
                let content_size = response.content_length().ok_or("Failed to get content length")?;
                let mut downloaded = 0;
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.map_err(|e| e.to_string())?;
                    downloaded += chunk.len() as u64;
                    file.write_all(&chunk).await.map_err(|e| e.to_string())?;
                    progress_callback(&binary.name, downloaded, content_size, ProgressType::Download);
                }
                file.sync_all().await.map_err(|e| e.to_string())?;
                println!("Downloaded {} to {}", binary.name, temp_file_path.display());
                
                // 检查是否是.zst文件，如果是则异步解压缩
                if ImageVariant::check_zst_file(&temp_file_path).await.map_err(|e| e.to_string())? {
                    println!("Extracting {} file", temp_file_path.to_string_lossy());
                    
                    // 创建不带.zst后缀的输出路径
                    let output_path = file_path;
                    
                    // 异步打开zst文件进行解压缩
                    let source = tokio::fs::File::open(&temp_file_path).await.map_err(|e| e.to_string())?;
                    let file_size = source.metadata().await.map_err(|e| e.to_string())?.len();
                    let reader = BufReader::new(source);
                    
                    // 创建目标文件
                    let mut target = tokio::fs::File::create(&output_path).await.map_err(|e| e.to_string())?;
                    
                    // 使用async-compression解压缩
                    let mut decoder = ZstdDecoder::new(reader);
                    
                    // 读取解码后的数据并写入目标文件
                    const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
                    let mut extracted = 0u64;
                    let mut buffer = vec![0u8; CHUNK_SIZE];
                    
                    // 由于我们无法提前知道解压后的大小，我们使用压缩文件的大小的估计值
                    // 通常，压缩比在2-5倍之间，我们选择3作为估计
                    let estimated_total = file_size * 3;
                    
                    loop {
                        let bytes_read = decoder.read(&mut buffer).await.map_err(|e| e.to_string())?;
                        if bytes_read == 0 {
                            break;
                        }
                        target.write_all(&buffer[..bytes_read]).await.map_err(|e| e.to_string())?;
                        extracted += bytes_read as u64;
                        progress_callback(&binary.name, extracted, estimated_total, ProgressType::Extract);
                    }
                    
                    target.flush().await.map_err(|e| e.to_string())?;
                    println!("Extracted {} to {}", binary.name, output_path.display());
                    
                    // 使用解压后的文件路径
                    binary.local_path = Some(output_path.to_string_lossy().to_string());
                } else {
                    // Rename the temp file to the final name
                    std::fs::rename(&temp_file_path, &file_path).map_err(|e| e.to_string())?;
                    println!("Renamed {} to {}", temp_file_path.to_string_lossy(), file_path.to_string_lossy());
                    binary.local_path = Some(file_path.to_string_lossy().to_string());
                }
            } else {
                return Err(format!("No web path or local path for binary: {}", binary.name));
            }
        }
        Ok(())
    }

    pub fn new(name: String, image_binarys: Vec<ImageBinary>) -> Self {
        let mut binaries = image_binarys;
        for binary in &mut binaries {
            let local_file_path = ImageVariant::get_local_path(&name, &binary.name);
            binary.local_path = if local_file_path.exists() {
                 Some(local_file_path.to_string_lossy().to_string())
            } else {
                None
            }
        }
        Self { name, image_binarys: binaries }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVersion {
    pub version: String,
    pub image_variants: Vec<ImageVariant>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_temp_dir() {
        let temp_dir = std::env::temp_dir();
        println!("Temp dir: {:?}", temp_dir);
    }

    #[tokio::test]
    async fn test_iscas_head_support() {
        let url = "https://fast-mirror.isrc.ac.cn/revyos/extra/images/lpi4a/20250323/boot-lpi4a-20250323_154524.ext4.zst";
        let response = reqwest::get(url).await.unwrap();
        assert_eq!(response.status(), 200, "Response should be OK");
        assert!(response.content_length().is_some(), "Content length should be available");
    }
    
    #[tokio::test]
    async fn test_download_binaries() {
        use mockito::{mock, server_url};
        
        // Setup mock server
        let mock_data = vec![1, 2, 3, 4, 5]; // Simple test data
        let mock_server = mock("GET", "/test/u-boot.bin")
            .with_status(200)
            .with_header("content-length", &mock_data.len().to_string())
            .with_body(mock_data.clone())
            .create();
        
        // Create an ImageVariant with a test binary
        let mut variant = ImageVariant {
            name: "test-variant".to_string(),
            image_binarys: vec![
                ImageBinary {
                    name: "u-boot.bin".to_string(),
                    web_path: Some(format!("{}/test/u-boot.bin", server_url())),
                    local_path: None,
                    binary_type: ImageBinaryType::UBoot,
                    hash_type: None,
                    hash_value: None,
                }
            ],
        };
        
        // Progress tracking variables to verify callback
        let mut received_progress = 0;
        let mut total_size = 0;
        
        // Execute the download with a simple callback that tracks progress
        let result = variant.download_binaries(|name, progress, total, _progress_type| {
            assert_eq!(name, "u-boot.bin");
            received_progress = progress;
            total_size = total;
        }).await;
        
        // Verify results
        assert!(result.is_ok(), "Download should succeed");
        assert_eq!(received_progress, mock_data.len() as u64, "Progress should match data size");
        assert_eq!(total_size, mock_data.len() as u64, "Total size should match data size");
        
        // Verify that local_path was set
        let binary = &variant.image_binarys[0];
        assert!(binary.local_path.is_some(), "Local path should be set after download");
        
        // Verify the downloaded file exists
        if let Some(path_str) = &binary.local_path {
            let path = std::path::Path::new(path_str);
            assert!(path.exists(), "Downloaded file should exist");
            
            // Clean up the test file
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
        
        mock_server.assert();
    }

    #[tokio::test]
    async fn test_download_binaries_iscas() {
        // Create an ImageVariant with a test binary
        let mut variant = ImageVariant {
            name: "test-variant".to_string(),
            image_binarys: vec![
                ImageBinary {
                    name: "boot-lpi4a-20250323_154524.ext4.zst".to_string(),
                    web_path: Some("https://fast-mirror.isrc.ac.cn/revyos/extra/images/lpi4a/20250323/boot-lpi4a-20250323_154524.ext4.zst".to_string()),
                    local_path: None,
                    binary_type: ImageBinaryType::UBoot,
                    hash_type: None,
                    hash_value: None,
                }
            ],
        };
        
        // Progress tracking variables to verify callback
        let mut total_size = 0;
        // Execute the download with a simple callback that tracks progress
        let result = variant.download_binaries(|_, _, total, _| {
            total_size = total;
        }).await;
        // Verify results
        assert!(result.is_ok(), "Download should succeed");
        
        // Verify that local_path was set
        let binary = &variant.image_binarys[0];
        assert!(binary.local_path.is_some(), "Local path should be set after download");
        
        // Verify the downloaded file exists
        if let Some(path_str) = &binary.local_path {
            let path = std::path::Path::new(path_str);
            assert!(path.exists(), "Downloaded file should exist");
            
            // Clean up the test file
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    #[tokio::test]
    async fn test_downloaded_binaries_detection() {
        use mockito::{mock, server_url};
        
        // Setup mock server
        let mock_data = vec![1, 2, 3, 4, 5]; // Simple test data
        let mock_server = mock("GET", "/test/u-boot.bin")
            .with_status(200)
            .with_header("content-length", &mock_data.len().to_string())
            .with_body(mock_data.clone())
            .create();
        
        // Create an ImageVariant with a test binary using ImageVariant::new
        let mut variant = ImageVariant::new(
            "test-variant".to_string(),
            vec![
            ImageBinary {
                name: "u-boot.bin".to_string(),
                web_path: Some(format!("{}/test/u-boot.bin", server_url())),
                local_path: None,
                binary_type: ImageBinaryType::UBoot,
                hash_type: None,
                hash_value: None,
            }
            ],
        );
        
        // Progress tracking variables to verify callback
        let mut received_progress = 0;
        let mut total_size = 0;
        
        // Execute the download with a simple callback that tracks progress
        let result = variant.download_binaries(|name, progress, total, _progress_type| {
            assert_eq!(name, "u-boot.bin");
            received_progress = progress;
            total_size = total;
        }).await;
        
        // Verify results
        assert!(result.is_ok(), "Download should succeed");
        assert_eq!(received_progress, mock_data.len() as u64, "Progress should match data size");
        assert_eq!(total_size, mock_data.len() as u64, "Total size should match data size");
        
        // Verify that local_path was set
        let binary = &variant.image_binarys[0];
        assert!(binary.local_path.is_some(), "Local path should be set after download");
        
        // Verify the downloaded file exists
        if let Some(path_str) = &binary.local_path {
            let path = std::path::Path::new(path_str);
            assert!(path.exists(), "Downloaded file should exist");
        }

        // Create a new ImageVariant with the same name and check if local_path is set
        let new_variant = ImageVariant::new(
            "test-variant".to_string(),
            vec![
                ImageBinary {
                    name: "u-boot.bin".to_string(),
                    web_path: Some(format!("{}/test/u-boot.bin", server_url())),
                    local_path: None,
                    binary_type: ImageBinaryType::UBoot,
                    hash_type: None,
                    hash_value: None,
                }
            ],
        );
        assert!(new_variant.image_binarys[0].local_path.is_some(), "Local path should be set after download");
        
        mock_server.assert();
    }
}