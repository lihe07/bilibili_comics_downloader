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
                    Arg::new("range")
                        .value_name("RANGE")
                        .long("range")
                        .short('r')
                        .help("指定下载范围，如1-3,5,7-")
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
                    Arg::new("format")
                        .value_name("FORMAT")
                        .short('f')
                        .long("format")
                        .help("导出的格式，epub | pdf | zip")
                )
                // .arg(
                //     Arg::new("from")
                //         .value_name("FROM")
                //         .long("from")
                //         .help("从第几话开始下载，默认为0")
                //         .required(false)
                // )
                // .arg(
                //     Arg::new("to")
                //         .value_name("TO")
                //         .long("to")
                //         .help("下载到第几话，默认为最后一话")
                //         .required(false)
                // )
                .arg(
                    Arg::new("range")
                        .value_name("RANGE")
                        .long("range")
                        .short('r')
                        .help("指定导出范围，如1-3,5,7-")
                )
                .arg(
                    Arg::new("split")
                        .help("是否每一话输出一个文件，不能与group同时使用")
                        .short('s')
                        .long("split")
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("OUTPUT")
                        .help("输出目录")
                        .required(false)
                )
                .arg(
                    Arg::new("group")
                        .long("group")
                        .short('g')
                        .value_name("GROUP")
                        .required(false)
                        .help("分组导出每组包含的章节数量")
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
                let grouping = matches.value_of("group").unwrap_or("0").parse::<usize>().unwrap();
                if grouping > 0 && split {
                    log.error("不能同时使用分组和拆分");
                    return;
                }
                let format = matches.value_of("format").unwrap();
                if format != "epub" && format != "pdf" && format != "zip" && format != "cbz" {
                    log.error("目前只支持导出 epub | pdf | zip 格式");
                    return;
                }
                lib::export(id_or_link.to_owned(), range, grouping, split, matches.value_of("output"), format.to_owned());
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
