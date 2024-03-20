use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;
use configparser::ini::Ini;
use windows_service_controller::dword::STATUS;

#[derive(Parser)]
#[command(name = "lers梦魔")]
#[command(author = "lers.fun")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A tool to update RDP library.", long_about = None)]
pub struct Config {
    /// 静默模式执行(不等待用户键入)
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    /// 使用指定的地址更新rdpwrap.ini
    #[arg(
        short,
        long,
        default_value = "https://cdn.jsdelivr.net/gh/loyejaotdiqr47123/rdpwrap@master/res/rdpwrap.ini"
    )]
    url: Option<String>,

    #[arg(
    short,
    long,
    default_value = "C:\\Program Files\\RDP Wrapper\\rdpwrap.ini",
    value_parser = parser_position
    )]
    /// 指定rdpwrap.ini的位置
    position: Option<PathBuf>,

    #[arg(long, default_value = "false")]
    /// 重启rdp服务，不检查更新
    pub reboot: bool,
}

/// 验证路径是否正确
pub fn parser_position(pos: &str) -> Result<PathBuf, String> {
    let config_file: &str = "rdpwrap.ini";
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

impl Config {
    /// 获取配置更新
    pub fn ini(req: &mut reqwest::blocking::Response) -> Result<Ini, Box<dyn Error>> {
        let mut text = String::new();
        req.read_to_string(&mut text)?;
        let mut ini = Ini::new();
        match ini.read(text) {
            Ok(_) => Ok(ini),
            Err(_) => return Err("转化ini失败".into()),
        }
    }

    pub fn download(&self) -> Result<reqwest::blocking::Response, Box<dyn Error>> {
        let url = &self.url.to_owned().unwrap();
        println!("正在获取最新配置:{}", url);
        let client = reqwest::blocking::Client::new();
        Ok(client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0")
            .header("Accept", "*/*")
            .timeout(Duration::from_secs(20))
            .send()?)
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
            println!("存在新版配置文件:{}", new_date);
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
    pub fn save_local(
        &mut self,
        config: &mut reqwest::blocking::Response,
    ) -> Result<(), Box<dyn Error>> {
        Config::close_service()?;
        let mut bytes: Vec<u8> = vec![];
        config.copy_to(&mut bytes)?;
        return match fs::write(self.position.to_owned().unwrap(), bytes) {
            Ok(_) => {
                Config::start_service()?;
                Ok(())
            }
            Err(e) => Err(format!("本地文件保存错误{e}").into()),
        };
    }

    /// 开启rdp服务
    pub fn start_service() -> Result<(), Box<dyn Error>> {
        println!("尝试开启rdp服务");
        let service1 = windows_service_controller::WindowsService::new("UmRdpService").unwrap();
        let service2 = windows_service_controller::WindowsService::new("TermService").unwrap();
        if service1
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_RUNNING)
        {
            println!("服务UmRdpService已在运行")
        } else if service1
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_START_PENDING)
        {
            println!("服务UmRdpService正在启动")
        } else {
            service1.start_service().unwrap();
        }
        if service2
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_RUNNING)
        {
            println!("服务TermService已在运行")
        } else if service2
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_START_PENDING)
        {
            println!("服务TermService正在启动")
        } else {
            service2.start_service().unwrap();
        }
        Ok(())
    }

    /// 关闭rdp服务
    pub fn close_service() -> Result<(), Box<dyn Error>> {
        println!("尝试关闭rdp服务");
        let service1 = windows_service_controller::WindowsService::new("UmRdpService").unwrap();
        let service2 = windows_service_controller::WindowsService::new("TermService").unwrap();
        if service1
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_STOPPED)
        {
            println!("服务UmRdpService未在运行")
        } else if service1
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_STOP_PENDING)
        {
            println!("服务UmRdpService正在关闭")
        } else {
            service1.stop_service().unwrap();
        }
        if service2
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_STOPPED)
        {
            println!("服务TermService未在运行")
        } else if service2
            .query_service_status()
            .unwrap()
            .eq(&STATUS::SERVICE_STOP_PENDING)
        {
            println!("服务TermService正在关闭")
        } else {
            service2.stop_service().unwrap();
        }
        Ok(())
    }
}
