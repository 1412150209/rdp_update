use std::io::Read;
use std::panic::PanicInfo;
use std::process::exit;

use clap::Parser;
use native_dialog::{MessageDialog, MessageType};

use rdp_update::Config;

#[cfg(windows)]
pub fn panic_handler(_panic_info: &PanicInfo) {
    let panic_message = format!(
        "众所周知,没有bug的程序不是好程序,所以这个程序是好程序\n\n错误：{}",
        _panic_info
    );

    let _ = MessageDialog::new()
        .set_title("Bug啦!")
        .set_text(&panic_message)
        .set_type(MessageType::Error)
        .show_alert()
        .unwrap();

    exit(-1);
}

fn exit_program(quiet: bool, code: i32) -> ! {
    // 是否静默
    if !quiet {
        println!("\n等待任意按键");
        let _ = std::io::stdin().lock().read_exact(&mut [0u8]);
    }
    exit(code);
}

fn main() {
    std::panic::set_hook(Box::new(panic_handler));
    let mut cli = Config::parse();
    // 是否重启模式
    if cli.reboot {
        Config::close_service().unwrap_or_else(|err| {
            eprintln!("{}", err.to_string());
        });
        Config::start_service().unwrap_or_else(|err| {
            eprintln!("{}", err.to_string());
        });
        let message = "重启rdp结束,请在控制台查看结果\n\n".to_owned();
        let _ = MessageDialog::new()
            .set_title("重启完毕!")
            .set_text(&message)
            .set_type(MessageType::Info)
            .show_alert()
            .unwrap();
        exit(0)
    }
    // 获取新配置
    let origin = cli.download().unwrap_or_else(|err| {
        eprintln!("{}", err.to_string());
        exit_program(cli.quiet, -1);
    });
    // 获取本地配置
    let local_config = cli.get_local().unwrap_or_else(|err| {
        eprintln!("{}", err.to_string());
        exit_program(cli.quiet, -1);
    });
    let new_config = Config::ini(&origin).unwrap_or_else(|err| {
        eprintln!("{}", err.to_string());
        exit_program(cli.quiet, -1);
    });
    // 比较日期
    if Config::compare_date(&new_config, &local_config).unwrap_or_else(|err| {
        eprintln!("{}", err.to_string());
        exit_program(cli.quiet, -1);
    }) {
        // 需要更新
        cli.save_local(&origin).unwrap_or_else(|err| {
            eprintln!("{}", err.to_string());
            exit_program(cli.quiet, -1);
        });
        println!("更新成功");
    } else {
        println!("无需更新");
        Config::check_service().unwrap_or_else(|err| {
            eprintln!("{}", err.to_string());
            exit_program(cli.quiet, -1);
        })
    }
    exit_program(cli.quiet, 0)
}
