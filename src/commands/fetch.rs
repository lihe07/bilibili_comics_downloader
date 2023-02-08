use crate::config::CONFIG;

#[derive(Debug)]
enum Msg {
    Size(usize),
    Halt,
    Done,
}

async fn run_task(
    config: &Config,
    ep: &EpisodeInfo,
    ep_cache: Option<EpisodeCache>,
    ep_root: &PathBuf,
    statics_sender: &Sender<Msg>,
    bar: &ProgressBar,
) -> Option<()> {
    // 获取某个章节的图片索引
    let ep_cache = if let Some(ep_cache) = ep_cache {
        // ep_cache.paths = indexes.paths;
        // ep_cache.host = indexes.host;
        // ep_cache.sync(&ep_root);
        ep_cache
    } else {
        let indexes = network::get_episode_images(config, ep.id).await.unwrap();
        let ep_cache = EpisodeCache {
            id: ep.id,
            title: ep.title.to_owned(),
            files: vec![],
            paths: indexes.paths,
            host: indexes.host,
            ord: ep.ord,
            root_dir: ep_root.to_path_buf(),
        };
        ep_cache.sync(&ep_root);
        ep_cache
    };

    let not_downloaded = ep_cache.not_downloaded();
    let mut downloaded = Vec::new();

    for (i, url) in network::get_image_tokens(config, not_downloaded.clone())
        .await?
        .iter()
        .enumerate()
    {
        // 出错的概率很低，但不是没有
        let file_name = not_downloaded.get(i).unwrap().split('/').last().unwrap();
        let path = ep_root.join(file_name);
        if let Some(size) = down_to(config, url.to_owned(), &path).await {
            downloaded.push(file_name);
            statics_sender.send(Msg::Size(size)).await.unwrap();
        }
    }
    if not_downloaded.len() == downloaded.len() {
        bar.inc(1);
        Some(())
    } else {
        None
    }
}

pub async fn fetch(id_or_link: String, range: String) {
    let id = parse_id_or_link(id_or_link);
    let mut log = paris::Logger::new();
    let comic_info = network::get_comic_info(&config, id).await;
    let cache = cache::Cache::load(&config);
    let cache_root = Path::new(&config.cache_dir);
    if !cache_root.join(format!("{}", id)).is_dir() {
        std::fs::create_dir_all(cache_root.join(format!("{}", id))).unwrap();
    }
    let cover_path = &cache_root.join(format!("{}", id)).join("cover.jpg");

    let comic_cache = if let Some(comic) = cache.get_comic(id) {
        let mut comic = comic.clone();
        comic.title = comic_info.title.to_owned();
        comic
    } else {
        // 并没有这个漫画的缓存，则创建一个
        // 保存漫画封面
        cache::ComicCache {
            id,
            title: comic_info.title.to_owned(),
            episodes: HashMap::new(),
        }
    };
    if !cover_path.is_file()
        && (down_to(&config, comic_info.vertical_cover.clone(), cover_path).await).is_none()
    {
        log.error("漫画封面下载失败");
        exit(1);
    }
    comic_cache.sync(&cache_root.join(format!("{}", id)));
    // 获取全部可用章节

    let mut ep_list = comic_info.ep_list.clone();
    ep_list.retain(|ep| {
        if ep.is_locked {
            false
        } else if let Some(ep_cache) = comic_cache.get_episode(ep.id) {
            !ep_cache.not_downloaded().is_empty()
        } else {
            true
        }
    });
    ep_list = apply_range(ep_list, &range);
    if ep_list.is_empty() {
        log.warn("没有需要下载的章节");
        return;
    }

    ep_list.sort_by(|a, b| a.ord.partial_cmp(&b.ord).unwrap());
    log.info("将要下载的漫画章节：\n");
    let episodes: Vec<String> = ep_list
        .iter()
        .map(|ep| {
            let ep = ep.to_owned();
            format!("    {} - {}", ep.ord, ep.title)
        })
        .collect();
    println!("{}", episodes.join("\n"));
    // 这里可以多线程

    log.info("启动下载线程...");

    let style = indicatif::ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-");

    let bar_overall = ProgressBar::new(ep_list.len() as u64);
    bar_overall.set_style(style.clone());

    let (statics_sender, mut statics_receiver) = tokio::sync::mpsc::channel(10);
    let (halt_sender, mut halt_receiver) = tokio::sync::mpsc::channel(10);
    ctrlc::set_handler(move || {
        halt_sender.blocking_send(()).unwrap();
    })
    .expect("无法设置 ctrl+c 处理函数");

    let mut tasks = Vec::new();
    for ep in ep_list.iter() {
        let ep_root = cache_root
            .join(format!("{}", id))
            .join(format!("{}", ep.id));
        let statics_sender = statics_sender.clone();
        let bar = bar_overall.clone();
        let config = config.clone();
        let ep = ep.clone();
        tasks.push(tokio::task::spawn(async move {
            loop {
                let ep_cache = EpisodeCache::load(&ep_root);
                if let Some(()) =
                    run_task(&config, &ep, ep_cache, &ep_root, &statics_sender, &bar).await
                {
                    break;
                }
                bar.println(format!("任务 {} 失败! 3s后重试!", ep.id));
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }))
    }

    let bar = bar_overall.clone();
    tokio::task::spawn(async move {
        let mut last = (chrono::Utc::now(), 0);
        bar.set_message("计算下载速度...");
        loop {
            match statics_receiver.recv().await {
                Some(Msg::Halt) => {
                    bar.abandon();
                    log.warn("用户取消下载");
                    break;
                }
                Some(Msg::Done) => {
                    bar.finish();
                    log.success("下载完成");
                    break;
                }
                Some(Msg::Size(size)) => {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(last.0);
                    if duration.num_seconds() >= 1 {
                        let bytes_per_second = size as f64 / duration.num_seconds() as f64;
                        bar.set_message(format!(
                            "{} / s",
                            bytes_with_unit(bytes_per_second as u64)
                        ));
                        last = (now, size);
                    }
                }
                None => {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(last.0);
                    if duration.num_seconds() >= 1 {
                        bar.set_message(format!("{} / s", bytes_with_unit(0)));
                        last = (now, 0);
                    }
                }
            }
        }
    });

    let tasks = futures::future::join_all(tasks);

    let future1 = async {
        tasks.await;
    };

    let future2 = async {
        halt_receiver.recv().await;
    };

    pin_mut!(future1);
    pin_mut!(future2);

    if let Either::Left(_) = futures::future::select(future1, future2).await {
        bar_overall.finish();
        statics_sender.send(Msg::Done).await.unwrap();
    } else {
        bar_overall.abandon();
        statics_sender.send(Msg::Halt).await.unwrap();
    }

    // 进行清理工作
}
