use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{base_bk_option, bk_config::BackupConfig};
use log::{error, info, warn};

///基于版本控制的备份模式，根据文件的哈希值判断是否需要备份，并保留指定数量的历史备份版本
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionMode {
    pub task_config: BackupConfig,
}

impl VersionMode {
    /// 创建整个备份计划
    /// 应当只使用这个create生成计划
    pub fn create(task: BackupConfig) -> Self {
        VersionMode { task_config: task }
    }
    /// 用于执行备份计划，根据配置信息进行备份操作。
    /// 在backup方法中，首先获取备份目录的路径，并根据文件的哈希值判断是否需要进行备份。
    /// 如果需要备份，则根据指定的版本保留数，删除早期备份。
    /// 接着根据备份路径获取备份目录，如果获取失败，则跳过备份等待下一个任务。
    /// 获取成功后，获取所有需要备份的文件路径，并根据路径创建相应的文件夹。
    /// 然后将文件复制到备份目录中，并记录备份大小。
    /// 最后将哈希值写入配置文件中，表示备份完成。
    pub fn backup(mut self, task_name: &String) {
        // 备份目录
        let mut backup_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // 获取hash
        match BackupConfig::get_hash(&self.task_config.path) {
            Ok(hash) => {
                if !self.task_config.backup_hashs.contains(&hash) {
                    self.backup_files(task_name, &hash, &mut backup_path);
                } else {
                    info!(
                        "{:#?}",
                        &(task_name.to_owned() + ":检查到无更新,等待下一个备份任务")
                    );
                }
            }
            Err(e) => {
                error!(
                    "{:#?}",
                    &(task_name.to_owned() + "计算hash时发生错误:" + e.to_string().as_str()),
                );
                return;
            }
        }
    }

    fn backup_files(self: &mut Self, task_name: &String, hash: &String, backup_path: &mut PathBuf) {
        info!(
            "{:#?}",
            &(task_name.to_owned() + ":当前任务使用版本控制模式,检查到有更新,开始备份")
        );

        let hash_len = self.task_config.backup_hashs.len() + 1;
        let total_len = &self.task_config.preserve_version + 1;

        if &hash_len >= &total_len {
            match base_bk_option::remove_first_version(
                &self.task_config.backup_path,
                &self.task_config.path_title,
                &self.task_config.preserve_version,
            ) {
                Ok(s) => {
                    *backup_path = s;
                    info!(
                        "{:#?}",
                        &(task_name.to_owned() + ":检查到历史版本过多,已删除早期备份")
                    );
                }
                Err(e) => {
                    error!(
                        "{:#?}",
                        &(task_name.to_owned()
                            + "删除早期备份时发生错误:"
                            + e.to_string().as_str()),
                    );
                    return;
                }
            }
        }

        if *backup_path == PathBuf::from(env!("CARGO_MANIFEST_DIR")) {
            match base_bk_option::get_backup_path_by_version(
                &self.task_config.backup_path,
                &self.task_config.path_title,
                &self.task_config.backup_hashs.len(),
                &self.task_config.preserve_version,
            ) {
                Ok(s) => *backup_path = s,
                Err(e) => {
                    error!(
                        "{:#?}",
                        &(task_name.to_owned()
                            + "获取备份路径时发生错误:"
                            + e.to_string().as_str()),
                    );
                    return;
                }
            }
        }

        if *backup_path == PathBuf::from(env!("CARGO_MANIFEST_DIR")) {
            warn!(
                "{:#?}",
                &(task_name.to_owned() + "检查hash有更新,但未获取备份路径,跳过等待下一个备份任务"),
            );
            return;
        }

        match base_bk_option::get_all_path(&self.task_config.path) {
            Ok(path_list) => {
                match base_bk_option::create_all_dir(
                    &path_list,
                    backup_path,
                    &self.task_config.path_title,
                ) {
                    Ok(()) => {
                        match base_bk_option::copy_file(
                            &path_list,
                            backup_path,
                            &self.task_config.path_title,
                        ) {
                            Ok(size) => {
                                info!(
                                    "{:#?}",
                                    &(task_name.to_owned()
                                        + ":备份完成，备份大小为["
                                        + &size.to_string()
                                        + "]MB"),
                                );

                                match self.task_config.set_hash(task_name, hash) {
                                    Ok(_) => {
                                        info!(
                                            "{:#?}",
                                            &(task_name.to_owned()
                                                + ":hash写入完成,备份目录为 "
                                                + &backup_path.as_os_str().to_str().unwrap()
                                                + ",等待下一个备份任务"),
                                        );
                                    }
                                    Err(e) => {
                                        error!(
                                            "{:#?}",
                                            &(task_name.to_owned()
                                                + "写入hash时发生错误:"
                                                + e.to_string().as_str()),
                                        );
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                error!(
                                    "{:#?}",
                                    &(task_name.to_owned()
                                        + "备份文件时发生错误:"
                                        + e.to_string().as_str()),
                                );
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "{:#?}",
                            &(task_name.to_owned()
                                + "创建备份文件夹时发生错误:"
                                + e.to_string().as_str()),
                        );
                        return;
                    }
                }
            }
            Err(e) => {
                error!(
                    "{:#?}",
                    &(task_name.to_owned() + "读取需备份文件时发生错误:" + e.to_string().as_str()),
                );
                return;
            }
        }
    }
}
