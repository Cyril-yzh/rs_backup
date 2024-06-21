use chrono::{DateTime, Local};
use log::{warn,error};
use serde::{Deserialize, Serialize};
use sha256;
use std::io::{Error, Read};
use std::{
    fs::{metadata, read_dir, File},
    io::Write,
    path::PathBuf,
};

///读取hash
///取 {path_name}_hash.yaml
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BackupConfig {
    /// 备份目的地
    ///
    /// 输入相对路径则在程序目录下Backup目录
    ///
    /// 输入绝对路径则根据绝对目录
    ///
    /// 目录不存在时自动创建
    ///
    /// 例：
    ///
    ///     C:/BACKUP
    ///
    ///     BACKUP
    pub backup_path: String,
    /// 需备份根目录
    ///
    /// 例：
    ///
    ///     Y:/XXX_Backup
    pub path: String,
    /// 需备份根目录名
    ///
    /// 例：
    ///
    ///     XXX_Backup
    pub path_title: String,
    /// # 初始值 backup_hashs: []
    pub backup_hashs: Vec<String>,
    /// 备份模式
    ///
    /// 1:动态目录模式
    ///
    /// 2:版本控制模式
    pub mode: usize,
    /// 仅在mode2中使用
    ///
    /// 表示一个备份任务应当保留几个版本
    pub preserve_version: usize,
    /// 在mode1和2中使用
    ///
    /// 表示一个备份任务应当保留几天
    pub save_days: usize,
}

impl BackupConfig {
    pub fn create(task_name: &String) -> Result<BackupConfig, Error> {
        let hash_path = BackupConfig::get_hash_path(task_name);
        if hash_path.is_file() {
            let mut file = File::open(hash_path).unwrap();
            let mut buf = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_) => {
                    let result: BackupConfig;
                    match serde_yaml::from_str(buf.as_str()) {
                        Ok(c) => {
                            result = c;
                            Ok(result)
                        }
                        Err(e) => {
                            error!(
                                "{:#?}",
                                &(String::from(":读取配置文件时发生错误:") + e.to_string().as_str()),
                            );
                            Err(Error::new(std::io::ErrorKind::Other, "读取配置文件时发生错误"))
                            // panic!("读取配置文件时发生错误");
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "{:#?}",
                        &(String::from(":读取配置文件时发生错误:") + e.to_string().as_str()),
                    );
                    Err(Error::from(e))
                }
            }
        } else {
            warn!(
                "{:#?}",
                &(String::from(":找不到配置文件,文件应为 ")
                    + hash_path.as_os_str().to_str().unwrap())
            );
            Err(Error::new(std::io::ErrorKind::Other, "找不到配置文件"))
        }
    }

    ///整个目录取Hash
    ///考虑Err -> rsmb::send_mail
    #[allow(unused)]
    pub fn get_hash(root_path: &String) -> Result<String, Error> {
        let mut path_list = vec![String::from(root_path)];
        let mut modified_list: Vec<String> = Vec::new();
        let mut start_index = 0;
        let mut list_len = 0;
        //取path list
        loop {
            list_len = path_list.len();
            for index in start_index..path_list.len() {
                let path1 = &path_list[index];
                if metadata(path1)?.is_dir() {
                    for child_dir in read_dir(&path1)? {
                        path_list.push(String::from(
                            child_dir?.path().as_os_str().to_str().expect(""),
                        ));
                    }
                }
                let path2 = &path_list[index];
                if metadata(path2)?.is_dir() {
                    for child_dir in read_dir(&path2)? {
                        let datetime: DateTime<Local> = child_dir?.metadata()?.modified()?.into();
                        modified_list.push(datetime.format("%Y-%m-%d %T").to_string());
                    }
                }
            }
            if list_len == start_index {
                break;
            }
            start_index = list_len;
        }

        let mut hash_str = String::new();
        //暂时这么写
        for p in path_list.iter() {
            hash_str += p;
        }
        for m in modified_list.iter() {
            hash_str += m;
        }
        ///返回hash
        let result = sha256::digest(hash_str);
        Ok(result)
    }
    //写入Hash
    #[allow(unused)]
    pub fn set_hash(&mut self, yaml_name: &String, hash: &String) -> Result<(()), Error> {
        let hash_path = BackupConfig::get_hash_path(yaml_name);
        let mut file = File::create(hash_path)?;
        if !self.backup_hashs.contains(&hash) {
            if &self.backup_hashs.len() == &self.preserve_version {
                //移除第一个并插入
                self.backup_hashs.remove(0);
                self.backup_hashs.push(hash.to_owned());
            } else {
                //插入
                self.backup_hashs.push(hash.to_owned());
            }
            let mut buf = serde_yaml::to_string(&self).unwrap();
            file.write_all(buf.as_bytes());
            Ok(())
        } else {
            // Err(Error::last_os_error())
            Err(Error::new(
                std::io::ErrorKind::Other,
                "已存在此Hash,无需进行写入",
            ))
        }
    }
    //内部函数 取hash存放地址
    fn get_hash_path(yaml_name: &String) -> PathBuf {
        let mut hash_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        hash_path.push("BackupConfig");
        hash_path.push(yaml_name.to_owned() + ".yaml");
        hash_path
    }
}
