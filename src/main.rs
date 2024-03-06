use std::io::Read;
use std::process::exit;

use clap::Parser;

use rdp_update::Config;

fn main() {
    let cli = Config::parse();
    // 处理参数
    if let Err(e) = cli.parse_config() {
        eprintln!("{}", e);
        exit(-1);
    }
    // 获取新配置
    let new_config = match cli.get_new() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}", e);
            exit(-1);
        }
    };
    // 获取本地配置
    let local_config = match cli.get_local() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}", e);
            exit(-1);
        }
    };
    // 比较日期
    if match Config::compare_date(&new_config, &local_config) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}", e);
            exit(-1);
        }
    } {
        // 需要更新
        if let Err(e) = cli.save_local(&new_config) {
            eprintln!("{}", e);
            exit(-1);
        }
    }
    // 是否静默
    if !cli.quiet {
        println!("\n等待任意按键");
        let _ = std::io::stdin().lock().read_exact(&mut [0u8]);
    }
}
