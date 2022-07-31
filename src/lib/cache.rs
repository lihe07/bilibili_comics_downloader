use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use super::config::Config;

#[derive(serde::Serialize, serde::Deserialize)]
struct EpisodeMeta {
    title: String,
    ord: f64,
    // 顺序号
    paths: Vec<String>,
    // 页码顺序
    host: String,
}

#[derive(Debug, Clone)]
pub struct EpisodeCache {
    pub id: u32,
    // episode id 作为文件夹名称
    pub title: String,
    pub files: Vec<String>,
    // 真实文件列表 xx.jpg, xx.jpg, xx.jpg, ...
    pub paths: Vec<String>,
    // 文件顺序
    pub host: String,
    pub ord: f64,
    pub root_dir: PathBuf,
}


// impl AsRef<EpisodeInfo> for EpisodeCache {
//     fn as_ref(&self) -> &EpisodeInfo {
//         EpisodeInfo {
//             title: self.title.to_owned(),
//             id: self.id,
//             is_locked: true,
//             ord: self.ord,
//         }.as_ref()
//     }
// }

impl crate::lib::HasOrd for &'_ EpisodeCache {
    fn ord(&self) -> f64 {
        self.ord
    }
}


impl EpisodeCache {
    pub fn load<T: AsRef<Path>>(path: T) -> Option<EpisodeCache> {
        let meta_path = path.as_ref().join("meta.toml");
        let mut meta_file = std::fs::File::open(&meta_path).ok()?;
        let mut buf = String::new();
        meta_file.read_to_string(&mut buf).ok()?;
        let meta: EpisodeMeta = toml::from_str(&buf).ok()?;
        let files = std::fs::read_dir(path.as_ref())
            .ok()?
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_file())
            .filter(|path| path.extension() == Some("jpg".as_ref()) || path.extension() == Some("png".as_ref()))
            .map(|path| path.file_name().unwrap().to_str().unwrap().to_string())
            .collect::<Vec<_>>();
        Some(EpisodeCache {
            id: path.as_ref().file_name()?.to_str()?.parse::<u32>().ok()?,
            title: meta.title,
            files,
            paths: meta.paths,
            host: meta.host,
            ord: meta.ord,
            root_dir: path.as_ref().to_path_buf(),
        })
    }
    pub fn sync<T: AsRef<Path>>(&self, path: T) {
        if (!path.as_ref().is_dir()) || (!path.as_ref().exists()) {
            std::fs::create_dir_all(path.as_ref()).unwrap();
        }
        // 写入 meta.toml
        let meta_path = path.as_ref().join("meta.toml");
        let mut meta_file = std::fs::File::create(&meta_path).unwrap();
        let meta = EpisodeMeta {
            title: self.title.clone(),
            ord: self.ord,
            paths: self.paths.clone(),
            host: self.host.clone(),
        };
        let meta_str = toml::to_string(&meta).unwrap();
        meta_file.write_all(meta_str.as_bytes()).unwrap();
    }

    pub fn not_downloaded(&self) -> Vec<String> {
        // 返回未下载的文件名
        let mut not_downloaded = Vec::new();
        for path in &self.paths {
            let end = path.split("/").last().unwrap();
            if !self.files.contains(&end.to_string()) {
                not_downloaded.push(path.to_owned());
            }
        }
        not_downloaded
    }

    pub fn get_paths(&self) -> Vec<PathBuf> {
        self.paths.iter().map(|link| {
            let file_name = link.split("/").last().unwrap();
            self.root_dir.join(file_name)
        }).collect()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ComicMeta {
    title: String,
    author_names: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ComicCache {
    pub id: u32,
    // 漫画id 作为文件夹名称
    pub title: String,
    pub author_names: Vec<String>,
    pub tags: Vec<String>,
    // 漫画标题
    pub episodes: HashMap<u32, EpisodeCache>,
}

impl ComicCache {
    pub fn load<T: AsRef<Path>>(path: T) -> Option<ComicCache> {
        let meta_path = path.as_ref().join("meta.toml");
        let mut meta_file = std::fs::File::open(meta_path).ok()?;
        let mut buf = String::new();
        meta_file.read_to_string(&mut buf).ok()?;
        let meta: ComicMeta = toml::from_str(&buf).ok()?;
        let mut episodes = HashMap::new();
        for entry in std::fs::read_dir(path.as_ref()).ok()? {
            let entry = entry.ok()?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let episode_id = entry_path.file_name()?.to_str()?.parse::<u32>().ok()?;
                let episode_cache = EpisodeCache::load(entry_path.to_owned());
                if let Some(episode_cache) = episode_cache {
                    episodes.insert(episode_id, episode_cache);
                } else {
                    // 清理无效文件夹
                    std::fs::remove_dir_all(entry_path).ok()?;
                }
            }
        }
        Some(ComicCache {
            id: path.as_ref().file_name()?.to_str()?.parse::<u32>().ok()?,
            title: meta.title,
            author_names: meta.author_names,
            tags: meta.tags,
            episodes,
        })
    }

    pub fn get_episode(&self, id: u32) -> Option<&EpisodeCache> {
        self.episodes.get(&id)
    }

    pub fn sync(&self, path: &Path) {
        if (!path.is_dir()) || (!path.exists()) {
            std::fs::create_dir_all(path).unwrap();
        }
        // 写入 meta.toml
        let meta_path = path.join("meta.toml");

        let mut meta_file = std::fs::File::create(&meta_path).unwrap();
        let meta = ComicMeta {
            title: self.title.clone(),
            author_names: self.author_names.clone(),
            tags: self.tags.clone(),
        };
        let meta_str = toml::to_string(&meta).unwrap();
        meta_file.write_all(meta_str.as_bytes()).unwrap();
        // 写入每个章节
        for (id, episode) in &self.episodes {
            let episode_path = path.join(id.to_string());
            episode.sync(episode_path);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub comics: HashMap<u32, ComicCache>,
}

impl Cache {
    pub fn load(config: &Config) -> Cache {
        let root_dir = Path::new(&config.cache_dir);
        if (!root_dir.exists()) || !root_dir.is_dir() {
            std::fs::create_dir_all(&root_dir).unwrap();
            return Cache {
                comics: HashMap::new()
            };
        }
        // 遍历文件夹
        let mut comics = HashMap::new();
        for entry in root_dir.read_dir().expect("无法读取缓存文件夹") {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(comic_cache) = ComicCache::load(&entry_path) {
                        comics.insert(comic_cache.id, comic_cache);
                    } else {
                        // 清理无效文件夹
                        std::fs::remove_dir_all(entry_path).unwrap();
                    }
                }
            }
        }
        Cache {
            comics
        }
    }
    pub fn get_comic(&self, id: u32) -> Option<&ComicCache> {
        self.comics.get(&id)
    }
    // pub fn sync(&self, config: &Config) {
    //     let root_dir = Path::new(&config.cache_dir);
    //     for (id, comic) in &self.comics {
    //         comic.sync(&root_dir.join(id.to_string()));
    //     }
    // }
}

