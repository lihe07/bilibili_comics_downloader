use clap::{Arg, Command};

mod lib;

#[tokio::main]
async fn main() {
    let mut log = paris::Logger::new();

    let cmd = Command::new("bcdown")
        .bin_name("bcdown")
        .subcommand_required(false)
        .subcommand(
            Command::new("login")
                .about("通过cookie或者二维码登录bilibili漫画，如未给出登录方式，则显示当前登录信息")
                .arg(
                    Arg::new("sessdata")
                        .short('s')
                        .long("sessdata")
                        .value_name("SESSDATA")
                        .help("Cookies中的SESSDATA，获取方法见Github")
                        .required(false)
                )
                .arg(
                    Arg::new("qrcode")
                        .short('q')
                        .long("qrcode")
                        .help("通过二维码登录")
                        .required(false)
                )
        )
        .subcommand(
            Command::new("info")
                .about("获取工具信息，包括配置信息和缓存信息")
        )
        .subcommand(
            Command::new("clear")
                .about("清除缓存")
        )
        .subcommand(
            Command::new("list")
                .about("获取本地缓存的漫画列表")
        )
        .subcommand(
            Command::new("search")
                .about("在bilibili漫画中查找某个漫画")
                .arg(
                    Arg::new("id_or_link")
                        .value_name("ID_OR_LINK")
                        .help("漫画的ID或者链接")
                )
        )
        .subcommand(
            Command::new("fetch")
                .about("保存某个漫画的全部可用章节到缓存目录，但不导出为pdf")
                .arg(
                    Arg::new("id_or_link")
                        .value_name("ID_OR_LINK")
                        .help("漫画的ID或者链接")
                )
                .arg(
                    Arg::new("from")
                        .value_name("FROM")
                        .help("从第几话开始下载，默认为0")
                        .required(false)
                )
                .arg(
                    Arg::new("to")
                        .value_name("TO")
                        .help("下载到第几话，默认为最后一话")
                        .required(false)
                )
        )
        .subcommand(
            Command::new("export")
                .about("导出某个漫画的全部可用章节到pdf文件")
                .arg(
                    Arg::new("id_or_link")
                        .value_name("ID_OR_LINK")
                        .help("漫画的ID或者链接")
                )
                .arg(
                    Arg::new("from")
                        .value_name("FROM")
                        .help("从第几话开始下载，默认为0")
                        .required(false)
                )
                .arg(
                    Arg::new("to")
                        .value_name("TO")
                        .help("下载到第几话，默认为最后一话")
                        .required(false)
                )
                .arg(
                    Arg::new("split_pdf")
                        .value_name("SPLIT_PDF")
                        .help("是否按照每一话输出一个PDF文件")
                        .short('s')
                        .long("split-pdf")
                )
                .arg(
                    Arg::new("output")
                        .value_name("OUTPUT")
                        .help("输出目录")
                        .required(false)
                )
        );
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("login", matches)) => {
            if matches.is_present("sessdata") && matches.is_present("qrcode") {
                log.error("只能选择一种登录方式");
                return;
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
                lib::show_login_info().await
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
                let from = matches.value_of("from").unwrap_or("0").parse::<f64>().unwrap();
                let to = matches.value_of("to").unwrap_or("-1").parse::<f64>().unwrap();
                lib::fetch(id_or_link.to_owned(), from, to).await;
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
                let from = matches.value_of("from").unwrap_or("0").parse::<f64>().unwrap();
                let to = matches.value_of("to").unwrap_or("-1").parse::<f64>().unwrap();
                let split = matches.is_present("split_pdf");
                lib::export(id_or_link.to_owned(), from, to, split, matches.value_of("output"));
            } else {
                log.error("缺少漫画的ID或者链接");
                log.info("使用bcdown export <ID_OR_LINK> 来导出漫画");
                log.info("有效的ID或者链接可以是：");
                println!("    1. https://manga.bilibili.com/detail/mc29911");
                println!("    2. mc29911");
                println!("    3. 29911");
            }
        }
        Some((_, _)) => {}
        None => {
            log.error("需要指定一个子命令!");
            log.info("例如：\n\tbcdown login -q\t使用二维码登录\n\tbcdown info\t查看工具信息\n\tbcdown clear\t清理下载缓存");
        }
    }
}
