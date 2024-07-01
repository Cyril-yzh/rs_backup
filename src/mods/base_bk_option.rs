use chrono::{DateTime, Duration, Local};
use std::ffi::OsStr;
use std::fs::{self, metadata, read_dir};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::vec;

#[allow(unused)]
/// 根据目标位置的源目录在目标位置创建所有目录
/// 会将所有目录一一对应保留
/// 不会复制权限
pub fn create_all_dir(
    from_dir_list: &Vec<String>,
    to_path_name: &PathBuf,
    backup_title: &String,
) -> Result<(), Error> {
    for path in from_dir_list.iter() {
        if metadata(path)?.is_dir() {
            let mut path_buf = to_path_name.clone();
            let mut flag = false;
            for component in Path::new(path).components() {
                if flag {
                    path_buf.push(component.as_os_str());
                }
                if component.as_os_str() == OsStr::new(backup_title) {
                    flag = true;
                }
            }
            fs::create_dir_all(&path_buf)?;
        }
    }
    Ok(())
}

#[allow(unused)]
/// 将文件从源目录复制到目标位置
/// 如果原文件不存在Result是文件不存在的err
/// 如果目标文件不存在会直接创建
/// 目标文件存在会被直接覆盖
/// 成功的Result是成功拷贝的MB
pub fn copy_file(
    from_dir_list: &Vec<String>,
    to_path_name: &PathBuf,
    backup_title: &String,
) -> Result<u64, Error> {
    let mut size = 0;
    for path in from_dir_list.iter() {
        if metadata(path)?.is_file() {
            let mut path_buf = to_path_name.clone();
            let mut flag = false;
            for component in Path::new(path).components() {
                if flag {
                    path_buf.push(component.as_os_str());
                }
                if component.as_os_str() == OsStr::new(backup_title) {
                    flag = true;
                }
            }
            size += fs::copy(&path, &path_buf)?;
        }
    }
    Ok(size / 1_048_576) // 将字节转换为MB
}

#[allow(unused)]
/// 获取指定根目录内的所有路径（文件和目录）
/// 返回整个目录的Vec<String>
/// 包括所有文件夹及文件的名字
pub fn get_all_path(root_path: &String) -> Result<Vec<String>, Error> {
    let mut path_list = vec![root_path.clone()];
    let mut start_index = 0;

    loop {
        let list_len = path_list.len();
        for index in start_index..list_len {
            let path = &path_list[index];
            if metadata(path)?.is_dir() {
                for child_dir in read_dir(path)? {
                    path_list.push(child_dir?.path().to_string_lossy().to_string());
                }
            }
        }
        if list_len == start_index {
            break;
        }
        start_index = list_len;
    }
    Ok(path_list)
}

#[allow(unused)]
/// 获取一定天数内修改的所有路径
/// 根据日期返回整个目录的Vec<String>
/// 包括所有文件夹及文件的名字
pub fn get_all_path_by_day(
    from_path: &String,
    to_path: &String,
    backup_title: &String,
    day: usize,
) -> Result<Vec<String>, Error> {
    let save_day = Local::now() - Duration::days(day as i64);
    let mut from_list = Vec::new();

    let mut directories = vec![from_path.clone()];
    while let Some(path) = directories.pop() {
        for entry in read_dir(&path)? {
            let entry = entry?;
            let path_str = entry.path().to_string_lossy().to_string();
            if entry.file_type()?.is_dir() {
                directories.push(path_str.clone());
                from_list.push(path_str);
            } else if entry.file_type()?.is_file() {
                let modified_time: DateTime<Local> = entry.metadata()?.modified()?.into();
                if modified_time > save_day {
                    from_list.push(path_str);
                }
            }
        }
    }

    let mut to_list = Vec::new();
    let mut directories = vec![to_path.clone()];
    while let Some(path) = directories.pop() {
        for entry in read_dir(&path)? {
            let entry = entry?;
            let path_str = entry.path().to_string_lossy().to_string();
            if entry.file_type()?.is_dir() {
                directories.push(path_str.clone());
                to_list.push(path_str);
            } else if entry.file_type()?.is_file() {
                let modified_time: DateTime<Local> = entry.metadata()?.modified()?.into();
                if modified_time > save_day {
                    to_list.push(path_str);
                }
            }
        }
    }

    let mut temp_list = Vec::new();
    for path in &to_list {
        let mut path_buf = PathBuf::from(from_path);
        let mut flag = false;
        for component in Path::new(path).components() {
            if component.as_os_str() == OsStr::new(backup_title) || flag {
                flag = true;
                path_buf.push(component.as_os_str());
            }
        }
        temp_list.push(path_buf.to_string_lossy().to_string());
    }

    let result_list = from_list
        .into_iter()
        .filter(|x| !temp_list.contains(x))
        .collect::<Vec<String>>();
    Ok(result_list)
}

#[allow(unused)]
/// 获取或创建备份路径,返回可用的备份目录
/// backup_name取config,
/// current_version取hashs.len()
pub fn get_backup_path(root_name: &String, backup_name: &String) -> Result<PathBuf, Error> {
    let mut backup_path = get_backup_base_path(root_name);
    backup_path.push(backup_name);

    fs::create_dir_all(&backup_path)?;
    // println!("index  : {:#?}", index);
    // println!("get_backup_path 创建dir : {:#?}", backup_path);
    Ok(backup_path)
}

pub fn get_backup_path_by_version(
    root_name: &String,
    backup_name: &String,
    current_version: &usize,
    total_version: &usize,
) -> Result<PathBuf, Error> {
    let mut index = *current_version;
    let mut backup_path = get_backup_base_path(root_name);
    backup_path.push(backup_name);

    loop {
        backup_path.push(format!("bk_version_{}", index));
        if !backup_path.is_dir() {
            break;
        }
        if index >= *total_version {
            remove_first_version(root_name, backup_name, total_version)?;
            index = *total_version;
        } else {
            index += 1;
        }
        backup_path.pop();
    }

    fs::create_dir_all(&backup_path)?;

    Ok(backup_path)
}

/// 删除最旧的备份版本并重命名后续版本
/// backup_name：存储文件夹名
/// version：应当保留版本数量
/// 返回备份dir
pub fn remove_first_version(
    root_name: &String,
    backup_name: &String,
    version: &usize,
) -> Result<PathBuf, Error> {
    let mut path1 = get_backup_base_path(root_name);
    path1.push(backup_name);

    let old_version_path = path1.join(format!("bk_version_0"));
    if old_version_path.is_dir() {
        fs::remove_dir_all(&old_version_path)?;
    }

    for index in 1..=*version {
        let old_path = path1.join(format!("bk_version_{}", index));
        let new_path = path1.join(format!("bk_version_{}", index - 1));
        if old_path.is_dir() {
            fs::rename(&old_path, &new_path)?;
        }
    }

    Ok(path1.join(format!("bk_version_{}", *version - 1)))
}

/// 删除指定根目录内的所有空目录
pub fn delete_all_empty_dir(root_path: &String) -> Result<bool, Error> {
    let mut is_empty = true;
    for entry in read_dir(root_path)? {
        let path = entry?.path();
        if metadata(&path)?.is_dir() {
            if delete_all_empty_dir(&path.to_string_lossy().to_string())? {
                fs::remove_dir_all(&path)?;
            } else {
                is_empty = false;
            }
        } else {
            is_empty = false;
        }
    }
    Ok(is_empty)
}

/// 删除指定天数内未修改的文件
pub fn delete_expired_file(root_path: &String, day: usize) -> Result<bool, Error> {
    let save_day = Local::now() - Duration::days(day as i64);
    let mut path_list = vec![root_path.clone()];
    let mut start_index = 0;

    loop {
        let list_len = path_list.len();
        for index in start_index..list_len {
            let path = &path_list[index];
            if metadata(path)?.is_dir() {
                for child_dir in read_dir(path)? {
                    path_list.push(child_dir?.path().to_string_lossy().to_string());
                }
            }
        }
        if list_len == start_index {
            break;
        }
        start_index = list_len;
    }

    for path in path_list.iter() {
        if metadata(path)?.is_file() {
            let modified_time: DateTime<Local> = metadata(path)?.modified()?.into();
            if save_day > modified_time {
                fs::remove_file(path)?;
            }
        }
    }

    Ok(true)
}

fn get_backup_base_path(root_name: &String) -> PathBuf {
    let path = Path::new(root_name);
    if path.is_dir() {
        return path.to_path_buf();
    }
    let mut base_path = PathBuf::from("BackupConfig");
    base_path.push(root_name);
    base_path
}
