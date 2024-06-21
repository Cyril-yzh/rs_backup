use std::{fs::File, path::PathBuf};

use log::error;
use serde::{Deserialize, Serialize};
use std::io::Read;
///读取Config
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub backup_paths: Vec<String>,
    pub backup_hour: usize,
    pub backup_day: i64,
    pub sleep_duration_ms: u64, // 添加时间间隔字段
}

impl Config {
    ///目前从config.yaml创建，未来考虑稳定性
    pub fn create() -> Config {
        let mut config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        config_path.push("Config.yaml");
    
        if config_path.is_file() {
            let mut file = File::open(config_path).unwrap();
            let mut buf = String::new();
            if let Err(e) = file.read_to_string(&mut buf) {
                error!("读取配置文件时发生错误: {:?}", e);
                panic!("读取配置文件时发生错误");
            }
    
            if let Ok(result) = serde_yaml::from_str::<Config>(&buf) {
                return result;
            } else {
                error!("解析配置文件时发生错误");
            }
        } else {
            error!("找不到配置文件,文件应为 {}", config_path.as_os_str().to_str().unwrap());
        }
    
        panic!("读取配置文件时发生错误");
    }
}
