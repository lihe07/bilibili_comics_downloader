pub async fn list() {
    let mut log = paris::Logger::new();
    let config = Config::load();
    let cache = cache::Cache::load(&config);
    for comic in cache.comics.values() {
        log.info(format!("{} - {}：", comic.id, comic.title));
        let mut episodes = comic.episodes.values().collect::<Vec<_>>();
        episodes.sort_by(|a, b| a.ord.partial_cmp(&b.ord).unwrap());
        let episodes = episodes
            .iter()
            .map(|e| {
                if e.not_downloaded().is_empty() {
                    format!("    {} - {} - {}", e.ord, e.title, "已下载".green())
                } else {
                    format!("    {} - {} - {}", e.ord, e.title, "未下载".red())
                }
            })
            .collect::<Vec<_>>();
        println!("{}", episodes.join("\n"));
    }
}
