use std::io::Read;
use std::process::exit;

use clap::Parser;

use rdp_update::Config;

fn main() {
    let cli = Config::parse();
    // 处理参数
    cli.parse_config().unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(-1);
    });
    // 获取新配置
    let new_config = cli.get_new().unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(-1);
    });
    // 获取本地配置
    let local_config = cli.get_local().unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(-1);
    });
    // 比较日期
    if Config::compare_date(&new_config, &local_config).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(-1);
    }) {
        // 需要更新
        cli.save_local(&new_config).unwrap_or_else(|err| {
            eprintln!("{}", err);
            exit(-1);
        })
    } else{
        println!("无需更新");
    }
    // 是否静默
    if !cli.quiet {
        println!("\n等待任意按键");
        let _ = std::io::stdin().lock().read_exact(&mut [0u8]);
    }
}
