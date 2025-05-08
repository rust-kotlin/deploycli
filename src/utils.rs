use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{fs, io};
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub uuid: String,
    pub name: String,
    pub description: String,
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

pub fn run_script(script_path: &Path) {
    // 在运行之前先完整显示脚本内容，等待用户输入y同意执行
    println!("{}", "Script content:".green().bold());
    let mut script_content = String::new();
    let mut file = fs::File::open(script_path).expect("Failed to open script file");
    file.read_to_string(&mut script_content)
        .expect("Failed to read script file");
    println!("{}", script_content);
    // 等待用户输入y再执行，否则退出
    println!("{}", "Do you want to execute this script? (y/n)".green().bold());
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let trimmed = input.trim();
    if !trimmed.eq_ignore_ascii_case("y") {
        println!("{}", "Script execution cancelled.".red().bold());
        return;
    }
    #[cfg(target_family = "unix")]
    {
        let mut child = std::process::Command::new("sh")
            .arg(script_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit()) // 直接继承父进程的 stdout
            .stderr(std::process::Stdio::inherit()) // 直接继承父进程的 stderr
            .spawn()
            .expect("Failed to execute script");
        // 获取脚本的 stdin
        if let Some(mut stdin) = child.stdin.take() {
            println!("{}", "Wait for input (type ':q' to quit):".red().bold());
            // 创建一个循环，持续监听用户输入
            let _ = std::thread::spawn(move || {
                let mut input = String::new();
                let stdin_handle = io::stdin();
                loop {
                    input.clear();
                    print!("{}", "> ".green().bold()); // 提示符
                    io::stdout().flush().unwrap(); // 刷新输出缓冲区
                    stdin_handle
                        .read_line(&mut input)
                        .expect("Failed to read input");
                    let trimmed = input.trim();
                    if trimmed.eq_ignore_ascii_case(":q") {
                        break; // 用户输入 "exit" 时退出循环
                    }
                    // 将用户输入写入脚本的 stdin
                    if let Err(e) = stdin.write_all(input.as_bytes()) {
                        eprintln!("Failed to write to stdin: {}", e);
                        break;
                    }
                }
            });
        }
        // 等待脚本执行完成
        let status = child.wait().expect("Failed to wait on child");
        if status.success() {
            println!("{}", "Script executed successfully.".green().bold());
        } else {
            eprintln!("{} {:?}", "Script execution failed with status".red().bold(), status);
        }
    }
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
