use chrono::{DateTime, Duration, Local};
use std::ffi::OsStr;
use std::path::Path;
use std::vec;
use std::{
    fs::{self, metadata, read_dir},
    io::Error,
    path::PathBuf,
};
#[allow(unused)]
/// 根据目标位置的源目录在目标位置创建所有目录
/// 会将所有目录一一对应保留
/// 不会复制权限
pub fn create_all_dir(
    from_dir_list: &Vec<String>,
    to_path_name: &PathBuf,
    backup_title: &String,
) -> Result<(), Error> {
    let mut path_buf;
    let mut flag;
    for path in from_dir_list.iter() {
        path_buf = to_path_name.to_owned();
        //对传入的Vec<String>做验证
        if metadata(path)?.is_dir() {
            flag = false;
            for components in PathBuf::from(&path).iter() {
                if flag == true {
                    path_buf.push(components);
                }
                if components == OsStr::new(backup_title) || flag == true {
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
    let mut path_buf;
    let mut flag;
    for path in from_dir_list.iter() {
        path_buf = to_path_name.to_owned();
        if metadata(path)?.is_file() {
            flag = false;
            for components in PathBuf::from(&path).iter() {
                if flag == true {
                    path_buf.push(components);
                }
                if components == OsStr::new(backup_title) || flag == true {
                    flag = true;
                }
            }
            // println!("from {:#?}  to {:#?}", &path, &path_buf);
            size += fs::copy(&path, &path_buf)?;
        }
    }
    let result: u64 = size / 1048576;
    Ok(result)
}

/// 获取指定根目录内的所有路径（文件和目录）
/// 返回整个目录的Vec<String>
/// 包括所有文件夹及文件的名字
#[allow(unused)]
pub fn get_all_path(root_path: &String) -> Result<Vec<String>, Error> {
    let mut path_list = vec![String::from(root_path)];
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
        }
        if list_len == start_index {
            break;
        }
        start_index = list_len;
    }
    Ok(path_list)
}
/// 获取一定天数内修改的所有路径
/// 根据日期返回整个目录的Vec<String>
/// 包括所有文件夹及文件的名字
#[allow(unused)]
pub fn get_all_path_by_day(
    from_path: &String,
    to_path: &String,
    backup_title: &String,
    day: usize,
) -> Result<Vec<String>, Error> {
    let save_day = Local::now() - Duration::days(day.try_into().unwrap());

    let mut temp_from_list = vec![String::from(from_path)];
    let mut from_list = vec![String::from(from_path)];

    let mut start_index = 0;
    let mut list_len = 0;
    let mut temp_path: String;
    //取from_list
    loop {
        list_len = temp_from_list.len();
        for index in start_index..temp_from_list.len() {
            let path1 = &temp_from_list[index];
            if metadata(path1)?.is_dir() {
                for child_dir in read_dir(&path1)? {
                    temp_path = String::from(child_dir?.path().as_os_str().to_str().expect(""));
                    temp_from_list.push(temp_path.to_owned());

                    if metadata(temp_path.to_owned())?.is_dir() {
                        from_list.push(temp_path.to_owned());
                    } else if metadata(temp_path.to_owned())?.is_file() {
                        let datetime: DateTime<Local> =
                            metadata(temp_path.to_owned())?.modified()?.into();
                        if datetime > save_day {
                            from_list.push(temp_path.to_owned());
                        }
                    }
                }
            }
        }
        if list_len == start_index {
            break;
        }
        start_index = list_len;
    }
    let mut temp_to_list = vec![String::from(to_path)];
    let mut to_list = vec![String::from(to_path)];
    start_index = 0;
    list_len = 0;
    //取to_list
    loop {
        list_len = temp_to_list.len();
        for index in start_index..temp_to_list.len() {
            let path1 = &temp_to_list[index];
            if metadata(path1)?.is_dir() {
                for child_dir in read_dir(&path1)? {
                    temp_path = String::from(child_dir?.path().as_os_str().to_str().expect(""));
                    temp_to_list.push(temp_path.to_owned());

                    if metadata(temp_path.to_owned())?.is_dir() {
                        to_list.push(temp_path.to_owned());
                    } else if metadata(temp_path.to_owned())?.is_file() {
                        let datetime: DateTime<Local> =
                            metadata(temp_path.to_owned())?.modified()?.into();
                        if datetime > save_day {
                            to_list.push(temp_path.to_owned());
                        }
                    }
                }
            }
        }
        if list_len == start_index {
            break;
        }
        start_index = list_len;
    }
    ///////把to_list的文件的起始目录转为from_path的目录//////
    // 用意是判断目标目录是否有某些原文件
    // 如：C:/Rust/rsbk/BackUp/AA/A/1.txt 转为 Y:/AA/A/1.txt
    let mut temp_list: Vec<String> = Vec::new();
    let mut path_buf: PathBuf;
    let mut flag;
    for path in to_list.iter() {
        path_buf = PathBuf::from(from_path);
        flag = false;
        for components in PathBuf::from(&path).iter() {
            if flag == true {
                path_buf.push(components);
            }
            if components == OsStr::new(backup_title) || flag == true {
                flag = true;
            }
        }
        temp_list.push(String::from(path_buf.as_os_str().to_str().expect("")));
    }
    //判断
    //let str_vec=from_list.iter().map(|x| x.to_string()).collect::<Vec<_>>();
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
    let mut backup_path: PathBuf = get_backup_base_path(root_name);
    backup_path.push(backup_name);

    //不管有没有，直接根据backup_path创建
    fs::create_dir_all(&backup_path)?;

    // println!("index  : {:#?}", index);
    // println!("get_backup_path 创建dir : {:#?}", backup_path);
    Ok(backup_path)
}

#[allow(unused)]
/// 获取或创建备份路径,返回可用的备份目录
/// backup_name取config,
/// current_version取hashs.len()
/// total_version取config
pub fn get_backup_path_by_version(
    root_name: &String,
    backup_name: &String,
    current_version: &usize,
    total_version: &usize,
) -> Result<PathBuf, Error> {
    let mut index = current_version.to_owned();
    let mut backup_path: PathBuf = get_backup_base_path(root_name);
    backup_path.push(backup_name);
    backup_path.push(String::from("bk_version_") + &index.to_string());

    //到1还没有就返回1
    loop {
        if &index > total_version {
            remove_first_version(root_name, backup_name, total_version);
        }
        if backup_path.is_dir() {
            index += 1;
            backup_path.set_file_name(String::from("bk_version_") + &index.to_string());
        } else {
            break;
        }
    }

    //不管有没有，直接根据backup_path创建
    fs::create_dir_all(&backup_path)?;

    // println!("index  : {:#?}", index);
    // println!("get_backup_path 创建dir : {:#?}", backup_path);
    Ok(backup_path)
}

#[allow(unused)]
/// 删除最旧的备份版本并重命名后续版本
/// backup_name：存储文件夹名
/// version：应当保留版本数量
/// 返回备份dir
pub fn remove_first_version(
    root_name: &String,
    backup_name: &String,
    version: &usize,
) -> Result<PathBuf, Error> {
    let mut index: usize = 1;
    let mut path1 = get_backup_base_path(root_name);
    path1.push(backup_name);
    path1.push(String::from("bk_version_") + &index.to_string());
    let mut path2 = path1.clone();

    //如果有第一个版本，删除
    if path1.is_dir() {
        fs::remove_dir_all(&path1)?;

        //然后一一改名,path1最后改成最大的目录
        loop {
            if &index == version {
                break;
            }
            path2.set_file_name(String::from("bk_version_") + &(index + 1).to_string());
            if path2.is_dir() {
                fs::rename(&path2, &path1);
            }
            index += 1;
            path1.set_file_name(String::from("bk_version_") + &index.to_string());
        }
    }
    //不管有没有，直接根据path1创建
    //无需验证错误，因为真有问题无法处理
    //后续改mail?
    fs::create_dir_all(&path1)?;
    // println!("创建dir : {:#?}", path1);
    Ok(path1)
}

/// 删除指定根目录内的所有空目录
#[allow(unused)]
pub fn delete_all_empty_dir(root_path: &String) -> Result<bool, Error> {
    // println!("root_path : {:#?}", root_path);
    let mut index = 0;
    let mut flag = false;
    if metadata(root_path)?.is_dir() {
        for (i, child_dir) in read_dir(&root_path)?.into_iter().enumerate() {
            // println!("child_dir : {:#?}", child_dir);
            let path = &child_dir?.path().to_owned();
            // println!("path {:#?}", &path);
            if metadata(path)?.is_dir() {
                flag = delete_all_empty_dir(&String::from(path.to_owned().to_str().expect("")))?;
                if flag {
                    fs::remove_dir_all(path)?;
                }
            }
            index = (i + 1);
        }
        if index == 0 {
            return Ok(true);
        } else {
            return Ok(false);
        }
    } else {
        return Ok(false);
    }
}

/// 删除指定天数内未修改的文件
#[allow(unused)]
pub fn delete_expired_file(root_path: &String, day: usize) -> Result<bool, Error> {
    let save_day = Local::now() - Duration::days(day.try_into().unwrap());
    let mut path_list = vec![String::from(root_path)];
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
        }
        if list_len == start_index {
            break;
        }
        start_index = list_len;
    }
    for path in path_list.iter() {
        if metadata(path.to_owned())?.is_file() {
            let datetime: DateTime<Local> = metadata(path.to_owned())?.modified()?.into();
            if save_day > datetime {
                fs::remove_file(path);
            }
        }
    }

    Ok(true)
}

fn get_backup_base_path(root_name: &String) -> PathBuf {
    if Path::new(root_name).is_dir() {
        return Path::new(root_name).to_path_buf();
    }
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(root_name);
    path
}
