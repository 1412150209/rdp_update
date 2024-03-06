use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Duration;

use clap::Parser;
use configparser::ini::Ini;

#[derive(Parser)]
#[command(name = "lers梦魔")]
#[command(author = "lers.fun")]
#[command(version = "0.2.0")]
#[command(about = "A tool to update RDP library.", long_about = None)]
pub struct Config {
    /// 静默模式执行(不等待用户键入)
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    /// 使用指定的地址更新rdpwrap.ini
    #[arg(
        short,
        long,
        default_value = "https://cdn.jsdelivr.net/gh/sebaxakerhtc/rdpwrap.ini@master/rdpwrap.ini"
    )]
    url: Option<String>,

    #[arg(
    short,
    long,
    default_value = "C:\\Program Files\\RDP Wrapper\\rdpwrap.ini",
    value_parser = Config::parser_position
    )]
    /// 指定rdpwrap.ini的位置
    position: Option<PathBuf>,

    #[arg(long, default_value = "false")]
    /// 重启rdp服务，不检查更新
    reboot: bool,
}

impl Config {
    /// 验证路径是否正确
    pub fn parser_position(pos: &str) -> Result<PathBuf, String> {
        let config_file: &str = "rdpwarp.ini";
        let path = Path::new(pos);
        if !Path::exists(path) {
            Err(format!("路径文件{pos}不存在."))
        } else {
            if path.is_dir() {
                let buf = path.join(config_file);
                if !buf.exists() {
                    Err(format!("路径{pos}下不存在配置文件{config_file}"))
                } else {
                    Ok(buf)
                }
            } else if path.is_file() {
                if !path.ends_with(config_file) {
                    Err(format!("配置文件应该为{config_file}"))
                } else {
                    Ok(path.to_path_buf())
                }
            } else {
                Err(format!("路径错误{pos}"))
            }
        }
    }

    /// 处理参数
    pub fn parse_config(&self) -> Result<(), Box<dyn Error>> {
        // 处理重启参数
        if self.reboot {
            Self::restart_service()?;
            exit(0)
        }
        Ok(())
    }

    /// 获取配置更新
    pub fn get_new(&self) -> Result<Ini, Box<dyn Error>> {
        let client = reqwest::blocking::Client::new();
        let new_config = client.get(&self.url.to_owned().unwrap())
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0")
            .header("Accept", "*/*")
            .timeout(Duration::from_secs(20))
            .send()?
            .text()?;
        let mut ini = Ini::new();
        match ini.read(new_config) {
            Ok(_) => Ok(ini),
            Err(_) => return Err("从网址读取配置失败".into()),
        }
    }

    /// 比较配置日期，判断是否需要更新
    pub fn compare_date(new: &Ini, local: &Ini) -> Result<bool, Box<dyn Error>> {
        let date = match new.get("Main", "Updated") {
            None => return Err("读取网址配置格式错误".into()),
            Some(date) => date,
        };
        let new_date = match chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return Err(format!("日期格式转换错误{e}").into()),
        };
        let date = match local.get("Main", "Updated") {
            None => return Err("读取本地配置格式错误".into()),
            Some(date) => date,
        };
        let local_date = match chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return Err(format!("日期格式转换错误{e}").into()),
        };
        if new_date.gt(&local_date) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 获取本地配置
    pub fn get_local(&self) -> Result<Ini, Box<dyn Error>> {
        let mut local = Ini::new();
        match local.load(self.position.to_owned().unwrap()) {
            Ok(_) => Ok(local),
            Err(e) => Err(format!("加载本地配置失败{e}").into()),
        }
    }

    /// 保存配置
    pub fn save_local(&self, new: &Ini) -> Result<(), Box<dyn Error>> {
        return match new.write(self.position.to_owned().unwrap()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("本地文件保存错误{e}").into()),
        };
    }
    /// 重启rdp服务
    pub fn restart_service() -> Result<(), Box<dyn Error>> {
        println!("尝试关闭rdp服务");
        let mut cmd = std::process::Command::new("cmd");
        match cmd.args(["/c", "echo Y | net stop UmRdpService"]).output() {
            Ok(_) => {}
            Err(_) => return Err("关闭UmRdpService失败,请尝试使用管理员权限启动".into()),
        }
        match cmd.args(["/c", "echo Y | net stop TermService"]).output() {
            Ok(_) => {}
            Err(_) => return Err("关闭TermService失败,请尝试使用管理员权限启动".into()),
        }
        println!("尝试启动rdp服务");
        match cmd.args(["/c", "echo Y | net start UmRdpService"]).output() {
            Ok(_) => {}
            Err(_) => return Err("启动UmRdpService失败,请尝试使用管理员权限启动".into()),
        }
        match cmd.args(["/c", "echo Y | net start TermService"]).output() {
            Ok(_) => {}
            Err(_) => return Err("启动TermService失败,请尝试使用管理员权限启动".into()),
        };
        Ok(())
    }
}
