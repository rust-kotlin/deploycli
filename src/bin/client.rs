use anyhow::anyhow;
use clap::{Parser, Subcommand};
use colored::Colorize;
use deploycli::{run_script, Task};
use deploycli::{create_zip, unpack_zip};
use reqwest::blocking::{Client, multipart};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::process;
use std::fs;

const CONFIG_PATH: &str = "/etc/deploycli/config.toml";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    server: String,
    password: String,
}

/// CLI client for task management
#[derive(Parser)]
#[command(
    name = "deploy",
    version = "1.0",
    author = "TomZz",
    about = "CLI client for task management"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new task
    New {
        /// Name of the new task
        name: String,
    },
    /// Get tasks or a specific task by index
    Get {
        /// Index of the task to get
        index: Option<usize>,
    },
    /// Upload a task
    Post {
        /// Path to the task file to upload
        path: String,
    },
    /// Delete a task
    Delete {
        /// Index of the task to delete
        index: usize,
    },
    /// Update remote database index
    Update,
    /// CLean local cache
    Clean {
        /// Index of the task to clean
        index: Option<usize>,
    },
}

fn main() {
    // 定义命令行参数
    let cli = Cli::parse();

    // 检查配置文件是否存在
    if !Path::new(CONFIG_PATH).exists() {
        create_default_config();
    }

    // 读取配置文件
    let config: Config = match read_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Error reading config: {}", err);
            process::exit(1);
        }
    };

    // 创建 HTTP 客户端
    let client = Client::new();

    // 解析命令并执行
    match cli.command {
        Commands::New { name } => {
            if let Err(e) = create_new_task(&name) {
                eprintln!("Error: Failed to create new task. Caused by: {e}");
                process::exit(1);
            }
        }
        Commands::Get { index } => {
            if index.is_none() {
                if let Err(e) = list_tasks(&client, &config) {
                    eprintln!("Error: Failed to list tasks. Caused by: {e}");
                    process::exit(1);
                }
                return;
            }
            let index = index.unwrap();
            if let Err(e) = get_task_by_index(&client, &config, index) {
                eprintln!("Error: Failed to get task. Caused by: {e}");
                process::exit(1);
            }
        }
        Commands::Post { path } => {
            if let Err(e) = upload_task(&client, &config, &path) {
                eprintln!("Error: Failed to upload task. Caused by: {e}");
                process::exit(1);
            }
        }
        Commands::Delete { index } => {
            if let Err(e) = delete_task(&client, &config, index) {
                eprintln!("Error: Failed to delete task. Caused by: {e}");
                process::exit(1);
            }
        }
        Commands::Update => {
            if let Err(e) = update_database(&client, &config) {
                eprintln!("Error: Failed to update database. Caused by: {e}");
                process::exit(1);
            }
        }
        Commands::Clean { index } => {
            if let Err(e) = clean_cache(&client, &config, index) {
                eprintln!("Error: Failed to clean cache. Caused by: {e}");
                process::exit(1);
            }
        }
    }
}

fn create_new_task(name: &str) -> anyhow::Result<()> {
    // 创建一个新的任务目录，输入config.toml和run.sh脚本
    fs::create_dir(name)?;
    let config_content = r#"uuid = "{uuid}"
name = "{name}"
description = "This is an example task"
"#;
    let uuid = uuid::Uuid::new_v4();

    let config_content = config_content
        .replace("{uuid}", &uuid.to_string())
        .replace("{name}", name);

    let config_path = format!("{}/config.toml", name);
    fs::write(&config_path, config_content)?;

    #[cfg(target_family = "unix")]
    {
        let run_script = r#"#!/bin/sh
echo "Running task..."
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
# 切换到脚本所在目录
cd "$SCRIPT_DIR"
# Add your task logic here
"#;
        let script_path = format!("{}/run.sh", name);
        fs::write(&script_path, run_script)?;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(target_family = "windows")]
    {
        let script_path = format!("{}/run.bat", name);
        let run_script = r#"@echo off
echo Running task...
REM Add your task logic here
"#;
        fs::write(&script_path, run_script)?;
    }
    Ok(())
}

fn create_default_config() {
    let default_config = Config {
        server: "http://localhost:3000".to_string(),
        password: "password".to_string(),
    };

    let config_dir = Path::new(CONFIG_PATH).parent().unwrap();
    fs::create_dir_all(config_dir).unwrap();
    let config_content = toml::to_string(&default_config).unwrap();
    fs::write(CONFIG_PATH, config_content).unwrap();

    println!("Default config created at {}", CONFIG_PATH);
}

fn read_config() -> anyhow::Result<Config> {
    let content = fs::read_to_string(CONFIG_PATH)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn list_tasks(client: &Client, config: &Config) -> anyhow::Result<()> {
    let url = format!("{}/tasks", config.server);
    let response = client
        .get(&url)
        .header("Authorization", &config.password)
        .send();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let tasks: Vec<Task> = resp
                    .json()
                    .map_err(|e| anyhow::anyhow!("Failed to get tasks: {}", e))?;
                for (i, task) in tasks.iter().enumerate() {
                    println!(
                        "{}: {} - {}",
                        i.to_string().blue().bold(),
                        task.name.cyan(),
                        task.description.custom_color((192, 192, 192))
                    );
                }
            } else {
                eprintln!("Error: {}", resp.text()?);
            }
        }
        Err(err) => eprintln!("Request failed: {}", err),
    }
    Ok(())
}

fn get_task_by_index(client: &Client, config: &Config, index: usize) -> anyhow::Result<()> {
    let url = format!("{}/tasks", config.server);
    let response = client
        .get(&url)
        .header("Authorization", &config.password)
        .send();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let tasks: Vec<Task> = resp.json()?;
                if let Some(task) = tasks.get(index) {
                    // 首先检查是否有缓存的任务压缩包
                    let cache_path = format!("/tmp/{}-{}.zip", task.name, task.uuid);
                    let md5 = md5::compute(fs::read(&cache_path).unwrap_or_else(|_| {
                        println!("Failed to read cached file\nStarting to download...");
                        vec![]
                    }));
                    let download_url = format!("{}/tasks/download", config.server);
                    let src_path = format!("{}.zip", task.name);
                    let mut file = fs::File::create(&src_path)?;
                    let mut download_resp = client
                        .post(download_url)
                        .form(&[
                            ("uuid", &task.uuid),
                            ("name", &task.name),
                            ("md5", &format!("{:x}", md5)),
                        ])
                        .header("Authorization", &config.password)
                        .send()?;
                    let mut check = true;
                    if download_resp.status() == reqwest::StatusCode::NOT_MODIFIED {
                        println!("Task {} is up to date, no need to download.", task.name);
                        check = false;
                    }
                    if check {
                        if !download_resp.status().is_success() {
                            return Err(anyhow::anyhow!(
                                "Failed to download task {}",
                                download_resp.text()?
                            ));
                        }
                        std::io::copy(&mut download_resp, &mut file)?;
                        // let content = download_resp.bytes()?;
                        // file.write_all(&content)?;
                        println!("Task downloaded: {}.zip", task.name);
                        // 压缩包拷贝到/tmp目录
                        fs::copy(&src_path, cache_path)?;
                    } else {
                        println!("Using cached task: {}", cache_path);
                        // 拷贝缓存的压缩包到当前目录
                        fs::copy(cache_path, &src_path)?;
                    }
                    // 解压到/tmp目录中
                    let dest_dir = format!("/tmp/{}-{}", task.name, task.uuid);
                    let unpack_res = unpack_zip(Path::new(&src_path), Path::new(&dest_dir));
                    // 删除压缩包
                    fs::remove_file(src_path).unwrap();
                    // 解压后运行其中的run.sh脚本
                    if unpack_res.is_ok() {
                        #[cfg(target_family = "unix")]
                        {
                            let script_path = Path::new(&dest_dir).join("run.sh");
                            run_script(&script_path);
                        }
                    } else {
                        eprintln!(
                            "Error: Failed to unpack zip file {}",
                            unpack_res.unwrap_err()
                        );
                    }
                } else {
                    eprintln!("Error: Task index out of range.");
                }
            } else {
                eprintln!("Error: {:#?}", resp.json::<Value>());
            }
        }
        Err(err) => eprintln!("Request failed: {}", err),
    }
    Ok(())
}

fn upload_task(client: &Client, config: &Config, path: &str) -> anyhow::Result<()> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Err(anyhow!("File not found at {path}"));
    }
    // 再检查目录下是否有config.toml文件
    let config_path = file_path.join("config.toml");
    if !config_path.exists() {
        return Err(anyhow!("config.toml not found in the task directory"));
    }
    // 读取config.toml文件中的name值
    let config_content = fs::read_to_string(&config_path)?;
    let task: Task = toml::from_str(&config_content)?;
    // 至少要有run.sh或者run.bat脚本
    let script_path = if cfg!(target_family = "unix") {
        file_path.join("run.sh")
    } else {
        file_path.join("run.bat")
    };
    if !script_path.exists() {
        return Err(anyhow!("run.sh or run.bat not found in the task directory"));
    }
    // 压缩成zip
    let zip_path = format!("{}.zip", task.name);
    create_zip(file_path, Path::new(&zip_path))?;
    let url = format!("{}/tasks/upload", config.server);
    let file = fs::File::open(&zip_path)?;
    let filename = format!("{}-{}", task.name, task.uuid);
    let part = multipart::Part::reader(file)
        .file_name(filename)
        .mime_str("application/zip")?;
    let form = multipart::Form::new().part("file", part);
    let response = client
        .post(&url)
        .header("Authorization", &config.password)
        .multipart(form)
        .send();
    // 删掉临时文件
    fs::remove_file(&zip_path).unwrap();
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("Task uploaded successfully.");
            } else {
                eprintln!("Error: {:#?}", resp.json::<Value>());
            }
        }
        Err(err) => eprintln!("Request failed: {}", err),
    }
    Ok(())
}

fn delete_task(client: &Client, config: &Config, index: usize) -> anyhow::Result<()> {
    let tasks: Vec<Task> = client
        .get(format!("{}/tasks", config.server))
        .header("Authorization", &config.password)
        .send()?
        .json()?;
    if index >= tasks.len() {
        return Err(anyhow!("Task index out of range"));
    }
    let task = &tasks[index];
    let url = format!("{}/tasks/delete", config.server);
    // Post请求删除任务
    let resp = client
        .post(&url)
        .header("Authorization", &config.password)
        .form(&[("uuid", &task.uuid), ("name", &task.name)])
        .send()?;
    if resp.status().is_success() {
        println!("Task {} deleted successfully.", task.name);
        // 删除缓存
        clean_cache(client, config, Some(index))?;
    } else {
        eprintln!("Error: {:#?}", resp.json::<Value>());
    }
    Ok(())
}

fn update_database(client: &Client, config: &Config) -> anyhow::Result<()> {
    let url = format!("{}/tasks/update", config.server);
    let resp = client
        .get(&url)
        .header("Authorization", &config.password)
        .send()?;
    if resp.status().is_success() {
        println!("Database updated successfully.");
    } else {
        eprintln!("Error: {:#?}", resp.json::<Value>());
    }
    Ok(())
}

fn clean_cache(client: &Client, config: &Config, index: Option<usize>) -> anyhow::Result<()> {
    let mut tasks: Vec<Task> = client
        .get(format!("{}/tasks", config.server))
        .header("Authorization", &config.password)
        .send()?
        .json()?;
    if index.is_some() && index.unwrap() >= tasks.len() {
        return Err(anyhow!("Task index out of range"));
    }
    if index.is_some() {
        // tasks里只保留index的任务
        let index = index.unwrap();
        tasks = vec![tasks[index].clone()];
    }
    // 删除所有缓存
    for task in tasks {
        let cache_path = format!("/tmp/{}-{}", task.name, task.uuid);
        let cache_path = Path::new(&cache_path);
        if cache_path.exists() {
            fs::remove_dir_all(cache_path)?;
            println!("Cache for task {} deleted successfully.", task.name);
        } else {
            println!("No cache found for task {}.", task.name);
        }
        let cache_zip = cache_path.with_extension("zip");
        if cache_zip.exists() {
            fs::remove_file(&cache_zip)?;
            println!("Cache zip for task {} deleted successfully.", task.name);
        } else {
            println!("No cache zip found for task {}.", task.name);
        }
    }
    Ok(())
}
