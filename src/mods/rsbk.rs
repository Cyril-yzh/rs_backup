use super::bk_config::BackupConfig;
use super::config::Config;
use super::{dynamic_mode::DynamicMode, version_mode::VersionMode};
use log::{error, warn};


pub struct RSBK {
    tasks: Vec<BackupMode>,
    pub conf: Config,
}

pub enum BackupMode {
    DynamicMode(DynamicMode),
    // DynamicMode(DynamicMode),
    VersionMode(VersionMode),
}


impl RSBK {
    pub fn init() -> Config {
         Config::create()
    }


    pub fn create() -> Self {
        let conf = Config::create();
        let mut tasks_conf: Vec<BackupConfig> = Vec::new();
        let mut tasks: Vec<BackupMode> = Vec::new();
        for path in conf.backup_paths.iter() {
            match BackupConfig::create(path) {
                Ok(c) => {
                    tasks_conf.push(c);
                }
                Err(_) => {
                    continue;
                }
            }
        }

        if tasks_conf.len() > 0 {
            for task in tasks_conf.into_iter() {
                if task.mode == 1 {
                    tasks.push(BackupMode::DynamicMode(DynamicMode::create(task)));
                } else if task.mode == 2 {
                    tasks.push(BackupMode::VersionMode(VersionMode::create(task)));
                } else {
                    warn!(
                        "{:#?}",
                        &String::from("设置备份模式时发生错误，枚举值应为1~2")
                    );
                    continue;
                }
            }
            if tasks.len() > 0 {
                RSBK { tasks, conf }
            } else {
                error!(
                    "{:#?}",
                    &String::from("所有备份计划模式错误,无法完成初始化")
                );
                panic!();
            }
        } else {
            error!("{:#?}", &String::from("找不到任何备份计划,无法完成初始化"));
            panic!();
        }
    }
    pub fn run(self) {
        for (i, task) in self.tasks.into_iter().enumerate() {
            match task {
                BackupMode::DynamicMode(t) => t.backup(&self.conf.backup_paths[i]),
                BackupMode::VersionMode(t) => t.backup(&self.conf.backup_paths[i]),
            }
        }
    }
}
