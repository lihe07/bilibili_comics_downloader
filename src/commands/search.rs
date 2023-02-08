pub async fn search(id: u32) {
    let mut log = paris::Logger::new();
    let config = Config::load();
    let mut comic_info = network::get_comic_info(&config, id).await;
    log.success(format!("漫画标题：{}", comic_info.title.bold()));
    log.success(format!(
        "漫画作者 / 出版社：{}",
        comic_info.author_name.join(",")
    ));
    log.success(format!("漫画标签：{}", comic_info.styles.join(",")));
    comic_info
        .ep_list
        .sort_by(|a, b| a.ord.partial_cmp(&b.ord).unwrap());

    let episodes: Vec<String> = comic_info
        .ep_list
        .iter()
        .map(|ep| {
            let ep = ep.to_owned();
            if ep.is_locked {
                format!("    {} - {} - {}", ep.ord, "锁定".red(), ep.title)
            } else {
                format!("    {} - {} - {}", ep.ord, "可用".green(), ep.title)
            }
        })
        .collect();
    log.success("漫画章节：\n");
    println!("{}", episodes.join("\n"));
}
