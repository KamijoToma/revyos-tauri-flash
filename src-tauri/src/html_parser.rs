use scraper::{Html, Selector};
use std::collections::HashMap;

use crate::image::{ImageBinary, ImageBinaryType, ImageVariant, ImageVersion};

/// This function fetches HTML content from a given URL and parses it to extract links.
///
/// It returns a vector of hash maps, where each map contains the name and address of a link.
///
/// :url: The URL to fetch the HTML content from.
///
/// :returns: A vector of hash maps containing the name and address of each link found in the HTML content.
/// This function will always return the raw link list, so it may be return a parent directory link.
///
/// ## Example
/// ```
/// let url = "https://mirror.iscas.ac.cn/revyos/extra/images/";
/// let result = fetch_and_parse(url.to_string()).await;
/// println!("{:?}", result);
/// ```
pub async fn fetch_and_parse_raw(
    url: String,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    // PATCH: add a trailing slash to the URL if it doesn't have one, which make ISCAS Nginx happy
    let url = if url.ends_with('/') {
        url
    } else {
        format!("{}/", url)
    };
    let response = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&response);
    let a_selector = Selector::parse("a").unwrap();

    let mut links = Vec::new();
    for a in document.select(&a_selector) {
        if let Some(link) = a.value().attr("href") {
            let mut map = HashMap::new();
            map.insert("name".to_string(), a.text().collect::<Vec<_>>().join(""));
            map.insert("address".to_string(), link.to_string());
            links.push(map);
        }
    }
    Ok(links)
}

/// This function fetches HTML content from a given URL and parses it to extract links,
/// excluding the parent directory link.
pub async fn fetch_and_parse(
    url: String,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    // call fetch_and_parse_raw and filter out the parent directory link
    let mut links = fetch_and_parse_raw(url).await?;
    links.retain(|link| {
        !link
            .get("name").is_some_and(|name| name.contains("Parent directory") || name.contains("../"))
    });
    Ok(links)
}


async fn fetch_and_parse_lpi4a_image(url: String) -> Result<ImageVersion, Box<dyn std::error::Error>> {
    let result = fetch_and_parse(url.clone()).await?;
    // turn into Vec<ImageBinary>
    let image_bin = result
        .iter()
        .map(|link| ImageBinary::try_from_hashmap(link, &url))
        .collect::<Result<Vec<_>, _>>()?;
    // Assemble the image binaries into a ImageVariant
    // Pattern: Each ImageVariant must and should only contain a ROOT, a BOOT, and a UBOOT binary.
    // The rest of the binaries are optional and can be ignored.
    let rootfs_binary = image_bin
        .iter()
        .find(|link| link.binary_type == ImageBinaryType::ROOT)
        .cloned()
        .ok_or("Missing ROOT binary")?;
    let boot_binary = image_bin
        .iter()
        .find(|link| link.binary_type == ImageBinaryType::BOOT)
        .cloned()
        .ok_or("Missing BOOT binary")?;
    let image_variants: Vec<_> = image_bin
        .iter()
        .filter(|link| link.binary_type == ImageBinaryType::UBoot)
        .map(|link| {
            ImageVariant::new(
                link.name.clone(),
                vec![rootfs_binary.clone(), boot_binary.clone(), link.clone()],
            )
        })
        .collect();
    let version = if url.ends_with('/') {
        url.split('/').nth_back(1).unwrap_or("Unknown").to_string()
    } else {
        url.split('/').last().unwrap_or("Unknown").to_string()
    };
    Ok(ImageVersion {
        version,
        image_variants,
    })
}

pub async fn fetch_and_parse_lpi4a_image_all(url: Option<String>) -> Result<Vec<ImageVersion>, Box<dyn std::error::Error>> {
    let url = url.unwrap_or_else(|| "https://mirror.iscas.ac.cn/revyos/extra/images/lpi4a/".to_string());
    let result = fetch_and_parse(url.clone()).await?;
    let mut image_versions = Vec::new();
    for link in &result {
        // Construct further link using link["address"] and url
        if let Some(address) = link.get("address") {
            let new_url = format!("{}{}", url, address);
            match fetch_and_parse_lpi4a_image(new_url).await {
                Ok(image_version) => {
                    image_versions.push(image_version);
                }
                Err(e) => {
                    eprintln!("Failed to fetch and parse image version: {}", e);
                }
            }
        } else {
            eprintln!("Missing 'address' in link: {:?}", link);
        }
    }
    Ok(image_versions)
}

#[cfg(test)]
mod tests {
    use crate::image::{ImageBinary, ImageBinaryType};

    use super::*;

    #[tokio::test]
    async fn test_fetch_and_parse_raw() {
        let url = "https://mirror.iscas.ac.cn/revyos/extra/images/";
        let result = fetch_and_parse_raw(url.to_string()).await;
        match result {
            Ok(links) => {
                assert!(!links.is_empty(), "The links list should not be empty.");
                for link in &links {
                    println!("{:?}", link);
                }
                // Assert it contains a link whose name contains "Parent directory"
                let parent_dir_link = links.iter().find(|link| {
                    link.get("name").is_some_and(|name| name.contains("Parent directory") || name.contains("../"))
                });
                assert!(
                    parent_dir_link.is_some(),
                    "The links list should contain a link with 'Parent directory' in its name."
                );
            }
            Err(e) => panic!("Error occurred: {}", e),
        }
    }

    #[tokio::test]
    async fn test_fetch_and_parse() {
        let url = "https://mirror.iscas.ac.cn/revyos/extra/images/";
        let result = fetch_and_parse(url.to_string()).await;
        match result {
            Ok(links) => {
                assert!(!links.is_empty(), "The links list should not be empty.");
                for link in &links {
                    println!("{:?}", link);
                }
                // Assert it does not contain a link whose name contains "Parent directory"
                let parent_dir_link = links.iter().find(|link| {
                    link.get("name").is_some_and(|name| name.contains("Parent directory"))
                });
                assert!(
                    parent_dir_link.is_none(),
                    "The links list should not contain a link with 'Parent directory' in its name."
                );
            }
            Err(e) => panic!("Error occurred: {}", e),
        }
        // test without trailing slash
        let url = "https://mirror.iscas.ac.cn/revyos/extra/images";
        let result = fetch_and_parse(url.to_string()).await;
        match result {
            Ok(links) => {
                assert!(!links.is_empty(), "The links list should not be empty.");
                for link in &links {
                    println!("{:?}", link);
                }
                // Assert it does not contain a link whose name contains "Parent directory"
                let parent_dir_link = links.iter().find(|link| {
                    link.get("name").is_some_and(|name| name.contains("Parent directory"))
                });
                assert!(
                    parent_dir_link.is_none(),
                    "The links list should not contain a link with 'Parent directory' in its name."
                );
            }
            Err(e) => panic!("Error occurred: {}", e),
        }
    }

    #[tokio::test]
    async fn test_fetch_directory() {
        let url = "https://mirror.iscas.ac.cn/revyos/extra/images/lpi4a";
        let result = fetch_and_parse(url.to_string()).await;
        match result {
            Ok(links) => {
                assert!(!links.is_empty(), "The links list should not be empty.");
                for link in &links {
                    println!("{:?}", link);
                }
            }
            Err(e) => panic!("Error occurred: {}", e),
        }
    }

    
    use rustpython_vm as vm;
    #[test]
    fn test_rustpython_vm() -> vm::PyResult<()> {
        vm::Interpreter::without_stdlib(Default::default()).enter(|vm| {
            let scope = vm.new_scope_with_builtins();
            let source = r#"print("Hello World!")"#;
            let code_obj = vm
                .compile(source, vm::compiler::Mode::Exec, "<embedded>".to_owned())
                .map_err(|err| vm.new_syntax_error(&err, Some(source)))?;
    
            vm.run_code_obj(code_obj, scope)?;
    
            Ok(())
        })
    }

    #[tokio::test]
    async fn test_fetch_and_parse_lpi4a_image() {
        let url = "https://fast-mirror.isrc.ac.cn/revyos/extra/images/lpi4a/20250323/".to_string();
        let result = fetch_and_parse(url.clone()).await.unwrap();
        println!("Result: {:?}", result);
        // turn into Vec<ImageBinary>
        let image_bin = result.iter().map(|link| {
            ImageBinary::try_from_hashmap(link, &url).unwrap()
        }).collect::<Vec<_>>();
        // print image_bin
        for link in &image_bin {
            println!("{:?}", link);
        }
        // Assemble the image binarys into a ImageVariant
        // Pattern: Each ImageVariant must and should only contain a ROOT, a BOOT, and a UBOOT binary.
        // The rest of the binarys are optional and can be ignored.
        let rootfs_binary = image_bin.iter().find(|link| link.binary_type == ImageBinaryType::ROOT).cloned().unwrap();
        let boot_binary = image_bin.iter().find(|link| link.binary_type == ImageBinaryType::BOOT).cloned().unwrap();
        let image_variants: Vec<_> = image_bin.iter().filter(|link| link.binary_type == ImageBinaryType::UBoot)
        .map(|link| {
            ImageVariant {
                name: link.name.clone(),
                image_binarys: vec![rootfs_binary.clone(), boot_binary.clone(), link.clone()],
            }
        }).collect();
        // print result
        for link in &image_variants {
            println!("{:?}", link);
        }
    }

    #[tokio::test]
    async fn test_image_version_parse() {
        let url = "https://mirror.iscas.ac.cn/revyos/extra/images/lpi4a/20250323/".to_string();
        let image_version1 = fetch_and_parse_lpi4a_image(url.clone()).await.unwrap();
        assert_eq!(image_version1.version, "20250323".to_string());
        // url without trailing slash
        let url = "https://mirror.iscas.ac.cn/revyos/extra/images/lpi4a/20250323".to_string();
        let image_version2 = fetch_and_parse_lpi4a_image(url.clone()).await.unwrap();
        assert_eq!(image_version2.version, "20250323".to_string());
    }

    #[tokio::test]
    async fn test_fetch_and_parse_lpi4a_image_all() {
        let image_versions = fetch_and_parse_lpi4a_image_all(None).await.unwrap();
        // print image_versions
        for version in &image_versions {
            println!("Date of images: {}", version.version);
        }
    }
}
