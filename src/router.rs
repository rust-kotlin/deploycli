use anyhow::anyhow;
use log::error;
use salvo::prelude::*;
use serde_json::json;
use std::fs;
use std::path::Path;

use crate::config::CFG;
use crate::db::DB;
use crate::result::AppResult;
use crate::utils::{create_zip, unpack_zip};
use crate::utils::Task;


#[handler]
async fn list_tasks() -> AppResult {
    let tasks = DB.get_all_tasks()?;
    Ok(tasks.into())
}

#[handler]
async fn download_task(req: &mut Request, res: &mut Response) -> AppResult {
    let task_name = req.form::<String>("name").await.ok_or(anyhow!("Task name not found"))?;
    let task_uuid = req.form::<String>("uuid").await.ok_or(anyhow!("Task UUID not found"))?;
    let task_md5 = req.form::<String>("md5").await.ok_or(anyhow!("Task md5 not found"))?;
    let task = format!("{}-{}", task_name, task_uuid);
    let task_dir = Path::new("./tasks").join(&task);
    // 检查任务目录是否存在
    if !task_dir.exists() || !task_dir.is_dir() {
        return Err(anyhow!("Task not found").into());
    }
    // 创建压缩包路径
    let zip_path = Path::new("./tasks").join(format!("{}.zip", task));
    create_zip(&task_dir, &zip_path)?;
    // 计算压缩包的md5值
    let md5 = md5::compute(tokio::fs::read(zip_path.clone()).await?);
    // 如果md5值相同，则不返回文件
    if task_md5 == format!("{:x}", md5) {
        // 删除压缩包
        tokio::fs::remove_file(zip_path).await.unwrap_or_else(|e| {
            error!("Failed to delete zip file: {}", e);
        });
        // 渲染一个特殊的status code回去方便客户端判断
        res.stuff(StatusCode::NOT_MODIFIED, "");
        return Ok(0.into());
    }
    // 提供下载
    res.send_file(&zip_path, req.headers()).await;
    // 删除压缩包
    tokio::fs::remove_file(zip_path).await.unwrap_or_else(|e| {
        error!("Failed to delete zip file: {}", e);
    });
    // 这个位置返回是没有意义的
    // 因为上面已经返回了文件
    // 约定一个值代表什么都不做
    // 返回一个特殊定义的数
    Ok(0.into())
}

#[handler]
async fn upload_task(req: &mut Request) -> AppResult {
    let file = req.file("file").await.ok_or(anyhow!("No file uploaded"))?;
    // 如果目标目录存在，删除它
    let dest_dir = Path::new("./tasks").join(file.name().ok_or(anyhow!("File name not found"))?);
    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir)?;
    }
    // 解压到 tasks 目录
    unpack_zip(file.path(), &dest_dir)?;
    // 解析目标里的config.toml
    let config_path = dest_dir.join("config.toml");
    if !config_path.exists() {
        return Err(anyhow!("config.toml not found").into());
    }
    let content = fs::read_to_string(&config_path)?;
    let task: Task = toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config.toml: {}", e))?;
    // 插入数据库
    DB.add_task(&task)?;
    Ok("upload successfully".into())
}

#[handler]
async fn delete_task(req: &mut Request) -> AppResult {
    let task_name = req.form::<String>("name").await.ok_or(anyhow!("Task name not found"))?;
    let task_uuid = req.form::<String>("uuid").await.ok_or(anyhow!("Task UUID not found"))?;
    let task = format!("{}-{}", task_name, task_uuid);
    let task_dir = Path::new("./tasks").join(&task);
    // 检查任务目录是否存在
    if !task_dir.exists() || !task_dir.is_dir() {
        return Err(anyhow!("Task not found").into());
    }
    // 删除任务目录
    fs::remove_dir_all(&task_dir)?;
    // 从数据库删除任务
    DB.delete_tasks(&task_uuid, &task_name)?;
    Ok("delete successfully".into())
}

#[handler]
async fn update_database() -> AppResult {
    DB.update()?;
    Ok(().into())
}

#[handler]
async fn auth_middleware(req: &mut Request, res: &mut Response) {
    let auth_header: Option<String> = req.header("Authorization");
    if let Some(auth_header) = auth_header {
        if auth_header != CFG.server.password {
            res.stuff(StatusCode::UNAUTHORIZED, Json(json!("Unauthorized")));
        }
    } else {
        res.stuff(
            StatusCode::UNAUTHORIZED,
            Json(json!("Missing Authorization header")),
        );
    }
}

pub fn create_router() -> Router {
    Router::new()
        .hoop(auth_middleware)
        .push(Router::with_path("/tasks").get(list_tasks))
        .push(Router::with_path("/tasks/download").post(download_task))
        .push(Router::with_path("/tasks/upload").post(upload_task))
        .push(Router::with_path("/tasks/delete").post(delete_task))
        .push(Router::with_path("/tasks/update").get(update_database))
}   