use core::time;
use log4rs;
use mods::rsbk::RSBK;
use std::thread;
pub mod mods;
use log::error;

fn main() {
    log_init();
    loop {
    let rsbk = RSBK::create();
        log::info!("备份开始运行.");
        rsbk.run();
        log::info!("备份运行完成。休眠30秒.");
        thread::sleep(time::Duration::from_millis(30000));
    }
}

pub fn log_init() {
    match log4rs::init_file("log4rs.yaml", Default::default()) {
        Ok(_) => (),
        Err(e) => {
            error!("获取log4rs配置文件时发生错误：{}", e);
            panic!("获取log4rs配置文件时发生错误");
        }
    }
}