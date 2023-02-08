pub fn export(
    id_or_link: String,
    range: String,
    grouping: usize,
    split_episodes: bool,
    export_dir: Option<&str>,
    format: String,
) {
    let mut log = paris::Logger::new();
    let id = parse_id_or_link(id_or_link);
    let config = Config::load();
    let cache = cache::Cache::load(&config);
    if let Some(comic_cache) = cache.get_comic(id) {
        log.info(format!("开始导出漫画：{}", comic_cache.title));
        log.info(format!(
            "导出质量：{}",
            if let Some(dpi) = config.dpi {
                format!("{}dpi", dpi)
            } else {
                "最佳".to_string()
            }
        ));
        let mut ep_list = comic_cache.episodes.values().collect::<Vec<_>>();

        ep_list.sort_by(|a, b| a.ord.partial_cmp(&b.ord).unwrap());
        ep_list = apply_range(ep_list, &range);
        if ep_list.is_empty() {
            log.error("没有可以导出的章节");
            return;
        }
        let ep_list = if split_episodes {
            ep_list.iter().map(|ep| Item::Single(ep)).collect()
        } else if grouping > 0 {
            make_groups(ep_list, grouping)
        } else {
            vec![Item::Group(ep_list)]
        };

        let out_dir = export_dir.unwrap_or(config.default_download_dir.as_str());
        let out_dir = Path::new(out_dir).join(&comic_cache.title);
        if !out_dir.exists() || !out_dir.is_dir() {
            std::fs::create_dir_all(&out_dir).unwrap();
        }

        let out = out_dir.display().to_string();
        let cover_path = Path::new(&config.cache_dir)
            .join(format!("{}", id))
            .join("cover.jpg");
        let format = if format == "pdf" {
            exports::PDF {}.into()
        } else if format == "epub" {
            exports::Epub {
                cover: if cover_path.is_file() {
                    let mut cover = std::fs::File::open(&cover_path).unwrap();
                    let mut buf = Vec::new();
                    cover.read_to_end(&mut buf).unwrap();
                    Some(buf)
                } else {
                    None
                },
            }
            .into()
        } else {
            exports::Zip {}.into()
        };
        exports::export(&comic_cache.title, ep_list, &config, &out_dir, &format);
        log.success(format!("漫画导出至: {}", out));
    } else {
        log.error("在本地缓存中找不到该漫画");
    }
}
