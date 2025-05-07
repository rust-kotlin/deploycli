use std::sync::{Arc, LazyLock};

use anyhow::anyhow;
use polodb_core::{CollectionT, Database, bson::doc};

use crate::utils::Task;

pub struct TaskDatabase {
    db: Arc<Database>,
}

impl TaskDatabase {
    /// 初始化数据库
    fn new(db_path: &str) -> Self {
        let db = Database::open_path(db_path).expect("Failed to open database");
        let db = Arc::new(db);
        TaskDatabase { db }
    }

    /// 检测数据目录并更新修改到数据库的调用函数
    pub fn update(&self) -> anyhow::Result<()> {
        // 检查数据目录是否存在
        if !std::path::Path::new("./tasks").exists() {
            return Err(anyhow!("Data directory does not exist"));
        }
        // 先获取所有任务
        let mut tasks = self.get_all_tasks()?;
        // 检查任务目录是否存在
        for entry in std::fs::read_dir("./tasks")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let config_path = path.join("config.toml");
                if config_path.exists() {
                    let content = std::fs::read_to_string(&config_path)?;
                    let task: Task = toml::from_str(&content)?;
                    self.add_task(&task)?;
                    // 从tasks中删除已存在的任务
                    tasks.retain(|t| t.uuid != task.uuid && t.name != task.name);
                }
            }
        }
        // tasks里剩下的就是数据库里但是文件里没有的任务
        for task in tasks {
            // 删除数据库里的任务
            self.delete_tasks(&task.uuid, &task.name)?;
        }
        Ok(())
    }

    /// 添加任务到数据库
    pub fn add_task(&self, task: &Task) -> anyhow::Result<()> {
        let collection = self.db.collection::<Task>("tasks");
        // 检查任务是否已存在
        let existing_task: Option<Task> =
            collection.find_one(doc! { "uuid": &task.uuid, "name": &task.name })?;
        if existing_task.is_some() {
            // 更新description
            collection.update_one(
                doc! { "uuid": &task.uuid, "name": &task.name },
                doc! { "$set": { "description": &task.description } },
            )?;
            return Ok(());
        }
        collection.insert_one(task)?;
        Ok(())
    }

    #[allow(unused)]
    /// 根据 UUID 获取任务
    pub fn get_task(&self, uuid: &str, name: &str) -> anyhow::Result<Task> {
        let collection = self.db.collection::<Task>("tasks");
        let task: Option<Task> = collection.find_one(doc! { "uuid": uuid, "name": name })?;
        if task.is_none() {
            return Err(anyhow!("Task not found"));
        }
        Ok(task.unwrap())
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> anyhow::Result<Vec<Task>> {
        let collection = self.db.collection::<Task>("tasks");
        let tasks = collection
            .find(doc! {})
            .run()?
            .collect::<polodb_core::Result<Vec<Task>>>()?;
        Ok(tasks)
    }

    /// 删除任务
    pub fn delete_tasks(&self, uuid: &str, name: &str) -> anyhow::Result<()> {
        let collection = self.db.collection::<Task>("tasks");
        let result = collection.delete_one(doc! { "uuid": uuid, "name": name })?;
        if result.deleted_count == 0 {
            return Err(anyhow!("Task not found"));
        }
        Ok(())
    }
}

/// 全局的 TaskDatabase 实例
pub static DB: LazyLock<TaskDatabase> = LazyLock::new(|| {
    TaskDatabase::new("tasks.db") // 初始化数据库路径
});
