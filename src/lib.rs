use clap::{App, Arg, ArgMatches, SubCommand};
use log::{error, info, warn};
use std::process::exit;

// Public modules
pub mod cache;
pub mod commands;
pub mod config;

// Private modules
mod exports;
mod network;
mod utils;

fn parse_id_or_exit(s: String) -> u32 {
    if let Some(id) = utils::parse_id(s) {
        id
    } else {
        error!("无效的ID或者链接");
        exit(1);
    }
}

pub async fn run(command: Option<(&str, &ArgMatches)>) {
    match command {
        Some(("login", matches)) => {
            if matches.is_present("sessdata") && matches.is_present("qrcode") {
                error!("不能同时使用两种登陆方式");
                exit(1);
            }

            if matches.is_present("sessdata") {
                if let Some(sessdata) = matches.value_of("sessdata") {
                    lib::login(lib::LoginMethod::SESSDATA(sessdata.to_string())).await
                } else {
                    log.error("缺少SESSDATA参数");
                    log.info("使用bcdown login -s <SESSDATA> 来登录");
                }
            } else if matches.is_present("qrcode") {
                lib::login(lib::LoginMethod::QRCODE).await
            } else {
            }
        }
        Some(("info", _)) => {
            lib::info().await;
        }
        Some(("clear", _)) => {
            lib::clear();
        }
        Some(("list", _)) => {
            lib::list().await;
        }
        Some(("search", matches)) => {
            if let Some(id_or_link) = matches.value_of("id_or_link") {
                lib::search(id_or_link.to_owned()).await;
            } else {
                log.error("缺少漫画的ID或者链接");
                log.info("使用bcdown search <ID_OR_LINK> 来搜索漫画");
                log.info("有效的ID或者链接可以是：");
                println!("    1. https://manga.bilibili.com/detail/mc29911");
                println!("    2. mc29911");
                println!("    3. 29911");
            }
        }
        Some(("fetch", matches)) => {
            if let Some(id_or_link) = matches.value_of("id_or_link") {
                let range = matches.value_of("range").unwrap_or("").to_string();
                lib::fetch(id_or_link.to_owned(), range).await;
            } else {
                log.error("缺少漫画的ID或者链接");
                log.info("使用bcdown fetch <ID_OR_LINK> 来保存漫画");
                log.info("有效的ID或者链接可以是：");
                println!("    1. https://manga.bilibili.com/detail/mc29911");
                println!("    2. mc29911");
                println!("    3. 29911");
            }
        }
        Some(("export", matches)) => {
            if let Some(id_or_link) = matches.value_of("id_or_link") {
                if !matches.is_present("format") {
                    log.error("缺少输出格式");
                    log.info("使用bcdown export <ID_OR_LINK> -f <FORMAT> 来输出漫画");
                    log.info("有效的输出格式可以是：");
                    println!("    1. pdf");
                    println!("    2. epub");
                    return;
                }
                // let from = matches.value_of("from").unwrap_or("-1").parse::<f64>().unwrap();
                // let to = matches.value_of("to").unwrap_or("-1").parse::<f64>().unwrap();

                let range = matches.value_of("range").unwrap_or("").to_string();
                let split = matches.is_present("split");
                let grouping = matches
                    .value_of("group")
                    .unwrap_or("0")
                    .parse::<usize>()
                    .unwrap();
                if grouping > 0 && split {
                    log.error("不能同时使用分组和拆分");
                    return;
                }
                let format = matches.value_of("format").unwrap();
                if format != "epub" && format != "pdf" && format != "zip" {
                    log.error("目前只支持导出 epub | pdf | zip 格式");
                    return;
                }
                lib::export(
                    id_or_link.to_owned(),
                    range,
                    grouping,
                    split,
                    matches.value_of("output"),
                    format.to_owned(),
                );
            } else {
                log.error("缺少漫画的ID或者链接");
                log.info("使用bcdown export <ID_OR_LINK> -f <FORMAT> 来导出漫画");
                log.info("有效的ID或者链接可以是：");
                println!("    1. https://manga.bilibili.com/detail/mc29911");
                println!("    2. mc29911");
                println!("    3. 29911");
            }
        }
        Some((_, _)) => {}
        None => {
            log.error("需要指定一个子命令!");
            log.info("例如：\n\tbcdown login -q\t使用二维码登录\n\tbcdown info\t查看工具信息\n\tbcdown clear\t清理下载缓存\n\tbcdown list\t查看本地漫画列表\n\tbcdown search mc29911\t搜索漫画\n\tbcdown fetch mc29911\t下载漫画\n\tbcdown export mc29911 -f epub\t导出本地漫画");
        }
    }
}
