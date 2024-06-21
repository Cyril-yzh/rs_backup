use core::time;
use log4rs;
use mods::rsbk::RSBK;
use std::thread;
pub mod mods;
use chrono::{DateTime, Duration, Local, TimeZone};
use log::error;

fn main() {
    log_init();

    let mut conf = RSBK::init();
    let mut run_time = calculate_run_time(&Local::now(), conf.backup_hour.try_into().unwrap());

    loop {
        let rsbk = RSBK::create();
        if rsbk.conf.backup_hour != conf.backup_hour {
            let now = Local::now();
            run_time = calculate_run_time(&now, rsbk.conf.backup_hour.try_into().unwrap());

            conf.backup_hour = rsbk.conf.backup_hour;
        }

        if Local::now() > run_time {
            run_time = run_time + Duration::days(rsbk.conf.backup_day);
            rsbk.run();
        }

        thread::sleep(time::Duration::from_millis(conf.sleep_duration_ms));
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

/* 用于计算备份任务的运行时间 */
fn calculate_run_time(now: &DateTime<Local>, backup_hour: u32) -> DateTime<Local> {
    match Local.with_ymd_and_hms(
        now.format("%Y").to_string().parse().unwrap(),
        now.format("%m").to_string().parse().unwrap(),
        now.format("%d").to_string().parse().unwrap(),
        backup_hour.try_into().unwrap(),
        0,
        0,
    ) {
        chrono::LocalResult::Single(t) => t,
        _ => {
            error!("获取备份时间发生错误,请检查配置文件");
            panic!();
        }
    }
}
