use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub uuid: String,
    pub name: String,
    pub description: String
}

pub fn create_zip(src_dir: &Path, zip_path: &Path) -> std::io::Result<()> {
    let file = fs::File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Zstd)
        .unix_permissions(0o755)
        .last_modified_time(zip::DateTime::default());

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            zip.start_file(name, options)?;
            let mut f = fs::File::open(&path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub fn unpack_zip(src_path: &Path, dest_dir: &Path) -> std::io::Result<()> {
    // 如果dest_dir不存在，则创建它
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir)?;
    }
    let file = fs::File::open(src_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = dest_dir.join(file.mangled_name());
        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_zip() {
        let src_dir = PathBuf::from("./tasks");
        let zip_path = PathBuf::from("./tasks.zip");
        // Create a zip file
        create_zip(&src_dir, &zip_path).unwrap();
        // Check if the zip file was created successfully
        assert!(zip_path.exists());
        // Test unzip
        let dest_dir = PathBuf::from("./unzipped_tasks");
        unpack_zip(&zip_path, &dest_dir).unwrap();
        // Check if the unzipped directory exists
        assert!(dest_dir.exists());
        // Clean up
        fs::remove_file(zip_path).unwrap();
        fs::remove_dir_all(dest_dir).unwrap();
    }
}