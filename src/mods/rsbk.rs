use super::bk_config::{BackupConfig, BackupMode};
use super::{incremental_mode::IncrementalMode, version_mode::VersionMode};
use chrono::{DateTime, Duration, Local, Timelike};
use log::{error, warn};
use std::collections::HashMap;
use std::fs::read_dir;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

lazy_static::lazy_static! {
    static ref NEXT_BACKUP_TIMES: Mutex<HashMap<String, DateTime<Local>>> = Mutex::new(HashMap::new());
}

pub struct RSBK {
    tasks: Vec<Arc<BackupModeWrapper>>,
}

pub enum BackupModeWrapper {
    IncrementalMode {
        task: Arc<Mutex<IncrementalMode>>,
        name: String,
    },
    VersionMode {
        task: Arc<Mutex<VersionMode>>,
        name: String,
    },
}

impl RSBK {
    pub fn create() -> Self {
        let mut config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        config_path.push("BackupConfig");

        let mut tasks: Vec<Arc<BackupModeWrapper>> = Vec::new();
        let mut next_backup_times = NEXT_BACKUP_TIMES.lock().unwrap();

        match Self::read_backup_configs(&config_path) {
            Ok(confs) => {
                for (config, file_name) in confs.iter() {
                    if config.is_effect {
                        let initial_time = parse_initial_backup_time(&config.initial_backup_time);

                        //更新下一次备份时间表
                        if !next_backup_times.contains_key(file_name) {
                            next_backup_times.insert(file_name.clone(), initial_time);
                        }
                     

                        match &config.options {
                            BackupMode::IncrementalMode { .. } => {
                                tasks.push(Arc::new(BackupModeWrapper::IncrementalMode {
                                    task: Arc::new(Mutex::new(IncrementalMode::create(
                                        config.clone(),
                                    ))),
                                    name: file_name.clone(),
                                }));
                            }
                            BackupMode::VersionMode { .. } => {
                                tasks.push(Arc::new(BackupModeWrapper::VersionMode {
                                    task: Arc::new(Mutex::new(VersionMode::create(config.clone()))),
                                    name: file_name.clone(),
                                }));
                            }
                        }
                    }
                }
                next_backup_times.retain(|k, _| confs.iter().any(|(_, name)| name == k));

                if !tasks.is_empty() {
                    RSBK { tasks }
                } else {
                    error!("所有备份计划模式错误或无效, 无法完成初始化");
                    panic!();
                }
            }
            Err(e) => {
                error!("读取备份配置时发生错误: {:?}", e);
                panic!();
            }
        }
    }

    fn read_backup_configs(config_path: &Path) -> Result<Vec<(BackupConfig, String)>, Error> {
        let mut configs = Vec::new();
        for entry in read_dir(config_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
                match BackupConfig::create(&path) {
                    Ok(config) => configs.push((config, file_name)),
                    Err(e) => {
                        warn!("读取配置文件 {:?} 时发生错误: {:?}", path, e);
                    }
                }
            }
        }
        Ok(configs)
    }

    pub fn run(self) {
        let mut handles = vec![];
        let now = Local::now();

        for task_arc in self.tasks {
            let task_clone = Arc::clone(&task_arc);

            let handle = thread::spawn(move || {
                let task = task_clone.as_ref();
                let name = match &*task {
                    BackupModeWrapper::IncrementalMode { name, .. } => name,
                    BackupModeWrapper::VersionMode { name, .. } => name,
                };

                log::info!("检查任务的备份时间: {}", name);

                let next_backup_times_lock = NEXT_BACKUP_TIMES.lock().unwrap();
                if let Some(next_backup_time) = next_backup_times_lock.get(name) {
                    log::info!("当前时间: {:?}. 下次备份时间: {:?}", &now, next_backup_time);
                    if &now >= next_backup_time {
                        log::info!("开始任务备份: {}", name);

                        match &*task {
                            BackupModeWrapper::IncrementalMode { task, name } => {
                                let task_lock = task.lock().unwrap();
                                task_lock.backup(name);
                            }
                            BackupModeWrapper::VersionMode { task, name } => {
                                let mut task_lock = task.lock().unwrap();
                                task_lock.backup(name);
                            }
                        }

                        let backup_interval_minutes = match &*task {
                            BackupModeWrapper::IncrementalMode { task, .. } => {
                                let task_lock = task.lock().unwrap();
                                task_lock.task_config.backup_interval_minutes
                            }
                            BackupModeWrapper::VersionMode { task, .. } => {
                                let task_lock = task.lock().unwrap();
                                task_lock.task_config.backup_interval_minutes
                            }
                        };
                        let next_time = now + Duration::minutes(backup_interval_minutes as i64);
                        drop(next_backup_times_lock); // 释放锁
                        let mut next_backup_times_mut = NEXT_BACKUP_TIMES.lock().unwrap();
                        next_backup_times_mut.insert(name.clone(), next_time);
                        log::info!("任务备份完成: {}. 下次备份时间: {:?}", name, next_time);
                    } else {
                        log::info!("当前任务无需备份: {}.", name);
                    }
                } else {
                    log::warn!("找不到任务的备份时间: {}", name);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.join() {
                error!("备份任务执行时发生错误: {:?}", e);
            }
        }
    }
}
fn parse_initial_backup_time(time_str: &str) -> DateTime<Local> {
    // 解析时间字符串 "mm:ss" 并返回对应的 DateTime<Local>
    let now = Local::now();
    let parts: Vec<&str> = time_str.split(':').collect();
    let hours: u32 = parts[0].parse().unwrap();
    let minutes: u32 = parts[1].parse().unwrap();
    now.with_hour(hours).unwrap().with_minute(minutes).unwrap()
}
