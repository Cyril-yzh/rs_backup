use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{base_bk_option, bk_config::BackupConfig};
use log::{error, info};

///基于基于动态目录的备份模式，根据文件的更新情况进行备份，并根据保存天数删除过期文件
#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicMode {
    pub task_config: BackupConfig,
}
impl DynamicMode {
    /// 创建整个备份计划
    /// 应当只使用这个create生成计划
    pub fn create(task: BackupConfig) -> Self {
        DynamicMode { task_config: task }
    }
    ///  用于执行备份计划，根据配置信息进行备份操作。
    /// 在backup方法中，首先获取备份根路径。
    /// 接着获取所有需要备份的文件路径，并判断是否有更新，如果有更新则进行备份操作。
    /// 备份操作包括创建备份文件夹、将文件复制到备份目录中，并记录备份大小。
    /// 备份完成后，根据保存天数删除过期文件，并删除空的备份文件夹。
    pub fn backup(self, task_name: &String) {
        //备份目录
        let backup_path: PathBuf;

        //获取备份根路径
        match base_bk_option::get_backup_path(
            &self.task_config.backup_path,
            &self.task_config.path_title,
        ) {
            Ok(path) => backup_path = path,
            Err(e) => {
                error!(
                    "{:#?}",
                    &(task_name.to_owned() + ":获取备份路径时发生错误:" + e.to_string().as_str()),
                );
                return;
            }
        }

        //获取hash
        match base_bk_option::get_all_path_by_day(
            &self.task_config.path,
            &String::from(backup_path.as_path().to_str().expect("")),
            &self.task_config.path_title,
            self.task_config.save_days.to_owned(),
        ) {
            Ok(path_list) => {
                //如果有更新开始备份
                if path_list.len() > 0 as usize {
                    info!(
                        "{:#?}",
                        &(task_name.to_owned() + ":当前任务使用动态目录模式,检查到有更新,开始备份")
                    );

                    //开始备份
                    match base_bk_option::create_all_dir(
                        &path_list,
                        &backup_path,
                        &self.task_config.path_title,
                    ) {
                        Ok(()) => {
                            match base_bk_option::copy_file(
                                &path_list,
                                &backup_path,
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
                                    // println!("copy success size : {:#?}", size);
                                    match base_bk_option::delete_expired_file(
                                        &String::from(backup_path.as_path().to_str().expect("")),
                                        self.task_config.save_days.to_owned(),
                                    ) {
                                        Ok(_) => {
                                            match base_bk_option::delete_all_empty_dir(
                                                &String::from(
                                                    backup_path.as_path().to_str().expect(""),
                                                ),
                                            ) {
                                                Ok(_) => {
                                                    info!(
                                                            "{:#?}",
                                                            &(task_name.to_owned()
                                                                .to_owned()
                                                                + ":删除超出保存时效的文件及空目录完成,备份目录为 ："
                                                                +&backup_path.as_os_str().to_str().unwrap()
                                                                + ",等待下一个备份任务")
                                                        );
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "{:#?}",
                                                        &(task_name.to_owned()
                                                            + ":删除空目录时发生错误:"
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
                                                    + ":删除超出保存时效的文件时发生错误:"
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
                                            + ":备份文件时发生错误:"
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
                                    + ":创建备份文件夹时发生错误:"
                                    + e.to_string().as_str()),
                            );
                            return;
                        }
                    }
                } else {
                    info!(
                        "{:#?}",
                        &(task_name.to_owned() + ":检查到无更新,等待下一个备份任务"),
                    );
                }
            }
            Err(e) => {
                error!(
                    "{:#?}",
                    &(task_name.to_owned() + ":获取备份文件时发生错误:" + e.to_string().as_str()),
                );
                return;
            }
        }
    }
}
