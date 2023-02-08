fn get_dir_size(path: &str) -> u64 {
    let mut size = 0;
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            size += entry.metadata().unwrap().len();
        } else if entry.file_type().unwrap().is_dir() {
            size += get_dir_size(entry.path().to_str().unwrap());
        }
    }
    size
}

/// 输出配置信息
pub async fn info() {
    let config = Config::load();
    let mut log = paris::Logger::new();
    log.info("bcdown 版本: 0.2.2");
    if let Some(user_info) = network::get_user_info(&config).await {
        log.info("登录信息有效！");
        log.info(format!("用户名：{}", user_info.name));
        log.info(format!("漫币余额：{}", user_info.coin));
    }
    log.info(format!("缓存目录：{}", config.cache_dir));
    log.info(format!(
        "缓存目录大小：{}",
        bytes_with_unit(get_dir_size(config.cache_dir.as_str()))
    ));
    log.info(format!("默认下载目录：{}", config.default_download_dir));
}
