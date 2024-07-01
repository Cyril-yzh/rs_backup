// use std::io::{self, ErrorKind};
// use std::process::{Command, Stdio};

// /// 关闭当前系统的所有网络接口
// pub fn shutdown_all_interfaces() -> io::Result<()> {
//     match detect_os() {
//         Ok(OsType::Linux) => shutdown_all_interfaces_linux(),
//         Ok(OsType::Windows) => shutdown_all_interfaces_windows(),
//         Err(err) => Err(io::Error::new(
//             ErrorKind::Other,
//             format!("Failed to detect OS: {}", err),
//         )),
//     }
// }

// /// 开启当前系统的所有网络接口
// pub fn startup_all_interfaces() -> io::Result<()> {
//     match detect_os() {
//         Ok(OsType::Linux) => startup_all_interfaces_linux(),
//         Ok(OsType::Windows) => startup_all_interfaces_windows(),
//         Err(err) => Err(io::Error::new(
//             ErrorKind::Other,
//             format!("Failed to detect OS: {}", err),
//         )),
//     }
// }

// /// 枚举类型表示操作系统类型
// enum OsType {
//     Linux,
//     Windows,
// }

// /// 检测当前操作系统类型
// fn detect_os() -> Result<OsType, String> {
//     let os = std::env::consts::OS;
//     match os {
//         "linux" => Ok(OsType::Linux),
//         "windows" => Ok(OsType::Windows),
//         _ => Err(format!("Unsupported OS: {}", os)),
//     }
// }

// /// Linux 平台关闭所有网络接口
// fn shutdown_all_interfaces_linux() -> io::Result<()> {
//     let output = Command::new("sudo")
//         .arg("ifconfig")
//         .arg("-a")
//         .arg("down")
//         .stdout(Stdio::inherit()) // 将子进程的输出重定向到父进程，方便调试
//         .stderr(Stdio::inherit())
//         .output()?;

//     if output.status.success() {
//         println!("Successfully shut down all interfaces on Linux");
//         Ok(())
//     } else {
//         let error_msg = String::from_utf8_lossy(&output.stderr);
//         Err(io::Error::new(
//             ErrorKind::Other,
//             format!("Failed to shut down interfaces on Linux: {}", error_msg),
//         ))
//     }
// }

// /// Linux 平台开启所有网络接口
// fn startup_all_interfaces_linux() -> io::Result<()> {
//     // 获取所有接口名称
//     let output = Command::new("ip")
//         .arg("link")
//         .arg("show")
//         .output()?;
    
//     if !output.status.success() {
//         let error_msg = String::from_utf8_lossy(&output.stderr);
//         return Err(io::Error::new(
//             ErrorKind::Other,
//             format!("Failed to get interface list on Linux: {}", error_msg),
//         ));
//     }
    
//     // 解析接口名称
//     let output_str = String::from_utf8_lossy(&output.stdout);
//     let interfaces: Vec<&str> = output_str.lines()
//         .filter_map(|line| extract_interface_name(line))
//         .collect();

//     // 逐个启用每个接口
//     for iface in &interfaces {
//         println!("Found interface: {}", iface);

//         // 启用接口
//         let result = Command::new("sudo")
//             .arg("ip")
//             .arg("link")
//             .arg("set")
//             .arg(iface)
//             .arg("up")
//             .stdout(Stdio::inherit())
//             .stderr(Stdio::inherit())
//             .output();

//         match result {
//             Ok(output) if output.status.success() => {
//                 println!("Successfully started up interface {}", iface);
//             }
//             Ok(output) => {
//                 let error_msg = String::from_utf8_lossy(&output.stderr);
//                 return Err(io::Error::new(
//                     ErrorKind::Other,
//                     format!("Failed to start up interface {}: {}", iface, error_msg),
//                 ));
//             }
//             Err(err) => {
//                 return Err(io::Error::new(
//                     ErrorKind::Other,
//                     format!("Failed to start up interface {}: {}", iface, err),
//                 ));
//             }
//         }
//     }

//     Ok(())
// }



// /// Windows 平台关闭所有网络接口
// fn shutdown_all_interfaces_windows() -> io::Result<()> {
//     let output = Command::new("netsh")
//         .arg("interface")
//         .arg("set")
//         .arg("interface")
//         .arg("name=*")
//         .arg("admin=disable")
//         .output()?;

//     if output.status.success() {
//         println!("Successfully shut down all interfaces on Windows");
//         Ok(())
//     } else {
//         let error_msg = String::from_utf8_lossy(&output.stderr);
//         Err(io::Error::new(
//             ErrorKind::Other,
//             format!("Failed to shut down interfaces on Windows: {}", error_msg),
//         ))
//     }
// }

// /// Windows 平台开启所有网络接口
// fn startup_all_interfaces_windows() -> io::Result<()> {
//     let output = Command::new("netsh")
//         .arg("interface")
//         .arg("set")
//         .arg("interface")
//         .arg("name=*")
//         .arg("admin=enable")
//         .output()?;

//     if output.status.success() {
//         println!("Successfully started up all interfaces on Windows");
//         Ok(())
//     } else {
//         let error_msg = String::from_utf8_lossy(&output.stderr);
//         Err(io::Error::new(
//             ErrorKind::Other,
//             format!("Failed to start up interfaces on Windows: {}", error_msg),
//         ))
//     }
// }

// /// 辅助函数：从 IP link show 命令输出中提取接口名称
// fn extract_interface_name(line: &str) -> Option<&str> {
//     if let Some(idx) = line.find(':') {
//         let iface = &line[..idx].trim();
//         if iface.ends_with(':') {
//             return Some(&iface[..iface.len() - 1]);
//         }
//     }
//     None
// }