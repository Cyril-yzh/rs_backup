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

linux 环境下部署并备份 linux 中文件时，先安装环境
sudo apt-get update
sudo apt-get install sshfs

然后创建目录
mkdir 10.251.2.10

连接
sshfs xxx@10.251.2.10:/home/xxx/backup1 /App/10.251.2.10/

linux 环境下部署并备份 windows 中文件时，在windows上共享文件夹
然后在 linux 安装环境
sudo apt-get update
sudo apt-get install cifs-utils

创建一个本地目录作为挂载点
sudo mkdir -p /mnt/windows_share
使用 mount.cifs 命令挂载 Windows 共享目录
sudo mount.cifs //Windows_IP/Share_Name /mnt/windows_share -o user=your_username,password=your_password
使用域用户
sudo mount.cifs //Windows_IP/Share_Name /mnt/windows_share -o user=domain\\username,password=your_password

自动挂载
如果你希望在系统启动时自动挂载共享目录，可以将其添加到 /etc/fstab 文件中。

创建和设置 credentials 文件
创建 credentials 文件：

sudo nano /etc/samba/credentials
在文件中添加用户名和密码：
username=Administrator
password=Ff123!@#
设置文件权限：
sudo chmod 600 /etc/samba/credentials
更新 /etc/fstab
将挂载信息添加到 /etc/fstab 文件中：
sudo nano /etc/fstab
//10.251.2.4/测试 /App/10.251.2.4 cifs credentials=/etc/samba/credentials,iocharset=utf8,sec=ntlmssp,vers=3.0 0 0
验证配置
sudo mount -a
