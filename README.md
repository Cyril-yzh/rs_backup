# rs_backup
rust备份工具

windows 环境下部署并备份 linux 中文件时，先安装环境
https://github.com/winfsp/winfsp/releases/
https://github.com/winfsp/sshfs-win/releases/

然后映射网络驱动器
# 根目录
\\sshfs.r\username@remote_ip!port\

# home 目录
\\sshfs\username@remote_ip!port\ 或 \\sshfs.r\username@remote_ip!port\home\username\