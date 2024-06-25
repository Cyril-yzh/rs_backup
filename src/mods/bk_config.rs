use chrono::{DateTime, Local};
use log::error;
use serde::{Deserialize, Serialize};
use sha256;
use std::collections::HashMap;
use std::io::{Error, Read};
use std::sync::Mutex;
use std::{
    fs::{metadata, read_dir, File, OpenOptions},
    io::Write,
    path::PathBuf,
};

lazy_static::lazy_static! {
    static ref NEXT_BACKUP_TIMES: Mutex<HashMap<String, DateTime<Local>>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "mode")]
pub enum BackupMode {
    IncrementalMode {
        /// 表示一个备份任务应当保留几天
        save_days: usize,
    },
    VersionMode {
        /// # 初始值 backup_hashs: []
        backup_hashs: Vec<String>,
        /// 表示一个备份任务应当保留几个版本
        preserve_version: usize,
    },
}

///读取hash
///取 {path_name}_hash.yaml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupConfig {
    // pub backup_schedule_name: String,
    /// 备份目的地
    /// 输入相对路径则在程序目录下Backup目录
    /// 输入绝对路径则根据绝对目录
    /// 目录不存在时自动创建
    pub destination_path: String,
    /// 需备份根目录
    pub source_path: String,
    /// 每次备份的间隔时间(分钟)
    pub backup_interval_minutes: usize,
    /// 首次备份的时间(mm:ss)
    pub initial_backup_time: String,
    pub is_effect: bool,
    /// 备份模式
    ///
    /// 1:增量备份模式
    ///
    /// 2:版本控制模式
    pub options: BackupMode,
}

impl BackupConfig {
    pub fn create(path: &PathBuf) -> Result<BackupConfig, Error> {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        serde_yaml::from_str(&buf).map_err(|e| {
            error!("读取配置文件时发生错误: {:?}", e);
            Error::new(std::io::ErrorKind::Other, "读取配置文件时发生错误")
        })
    }

    //内部函数 取hash存放地址
    pub fn get_hash_path(task_name: &str) -> PathBuf {
        let mut hash_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        hash_path.push("BackupConfig");
        hash_path.push(task_name.to_owned() + ".yaml");
        hash_path
    }

    ///整个目录取Hash
    pub fn get_hash(root_path: &str) -> Result<String, Error> {
        let mut path_list = vec![root_path.to_string()];
        let mut modified_list = Vec::new();

        while let Some(path) = path_list.pop() {
            if let Ok(metadata) = metadata(&path) {
                if metadata.is_dir() {
                    for entry in read_dir(&path)? {
                        let child_path = entry?.path().to_string_lossy().to_string();
                        path_list.push(child_path.clone());
                    }
                } else if metadata.is_file() {
                    let modified_time: DateTime<Local> = metadata.modified()?.into();
                    modified_list.push(modified_time.format("%Y-%m-%d %T").to_string());
                }
            }
        }

        let mut hash_str = path_list.join("");
        hash_str += &modified_list.join("");
        Ok(sha256::digest(hash_str))
    }

    //写入Hash
    pub fn set_hash(&mut self, yaml_name: &str, hash: &str) -> Result<(), Error> {
        let hash_path = BackupConfig::get_hash_path(yaml_name);
    
        // Open the file in append mode to avoid overwriting
        let mut file = OpenOptions::new().write(true).truncate(true).open(&hash_path)?;
    
        if let BackupMode::VersionMode {
            backup_hashs,
            preserve_version,
        } = &mut self.options
        {
            if !backup_hashs.contains(&hash.to_string()) {
                if backup_hashs.len() == *preserve_version {
                    backup_hashs.remove(0);
                }
                backup_hashs.push(hash.to_string());
    
                // Serialize the struct to YAML
                let yaml_str = match serde_yaml::to_string(self) {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to serialize BackupConfig to YAML: {:?}", e),
                        ));
                    }
                };
    
                file.write_all(yaml_str.as_bytes())?;
                Ok(())
            } else {
                Err(Error::new(
                    std::io::ErrorKind::Other,
                    "已存在此Hash,无需进行写入",
                ))
            }
        } else {
            Err(Error::new(
                std::io::ErrorKind::Other,
                "动态模式不需要写入Hash",
            ))
        }
    }

    /// 自动识别 source_path 中的路径标题
    pub fn detect_path_title(&self) -> Option<String> {
        // 通过分隔符 '/' 或 '\\' 获取最后一个路径段
        if let Some(sep_pos) = self.source_path.rfind('/') {
            Some(self.source_path[(sep_pos + 1)..].to_string())
        } else if let Some(sep_pos) = self.source_path.rfind('\\') {
            Some(self.source_path[(sep_pos + 1)..].to_string())
        } else {
            None
        }
    }
}
