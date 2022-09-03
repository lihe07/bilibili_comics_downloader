#![allow(clippy::upper_case_acronyms)]
use reqwest::header::HeaderMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Config {
    pub sessdata: String,
    pub cache_dir: String,
    pub default_download_dir: String,
    pub dpi: Option<f64>,
}

fn mkdir<T: AsRef<Path>>(path: T) {
    if std::fs::create_dir_all(&path).is_err() {
        let mut log = paris::Logger::new();
        log.error(format!("无法创建目录：{}", path.as_ref().display()));
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut log = paris::Logger::new();
        log.info("初始化默认配置");

        let (config_path, cache_dir, default_download_dir) =
            if let Some(user_dir) = directories::UserDirs::new() {
                if let Some(document_dir) = user_dir.document_dir() {
                    let config_path = document_dir.join("bcdown/config.toml");
                    let cache_dir = document_dir.join("bcdown/cache");
                    let tmp = document_dir.join("bcdown/download");
                    let default_download_dir = user_dir.download_dir().unwrap_or(tmp.as_path());
                    (config_path, cache_dir, default_download_dir.to_path_buf())
                } else {
                    log.warn("无法获取用户文档目录，将使用用户根目录");
                    let home_dir = user_dir.home_dir();
                    let config_path = home_dir.join(".bcdown/config.toml");
                    let cache_dir = home_dir.join(".bcdown/cache");
                    let tmp = home_dir.join(".bcdown/download");
                    let default_download_dir = user_dir.download_dir().unwrap_or(tmp.as_path());
                    (config_path, cache_dir, default_download_dir.to_path_buf())
                }
            } else {
                log.warn("无法定位用户目录！将使用当前工作目录作为工具根目录");
                let current_dir = std::env::current_dir().unwrap();
                let config_path = current_dir.join("config.toml");
                let cache_dir = current_dir.join("bcdown_cache");
                let default_download_dir = current_dir.join("bcdown_download");
                (config_path, cache_dir, default_download_dir)
            };

        log.info(format!("配置文件路径：{}", config_path.display()));
        log.info(format!("缓存目录：{}", cache_dir.display()));
        log.info(format!("默认下载目录：{}", default_download_dir.display()));
        mkdir(cache_dir.as_path());
        mkdir(default_download_dir.as_path());

        let config = Config {
            sessdata: "".to_string(),
            cache_dir: cache_dir.to_string_lossy().to_string(),
            default_download_dir: default_download_dir.to_string_lossy().to_string(),
            dpi: None,
        };

        let mut config_file = std::fs::File::create(config_path).unwrap();
        let config_toml = toml::to_string(&config).unwrap();
        config_file.write_all(config_toml.as_bytes()).unwrap();
        config
    }
}

impl Config {
    fn path() -> Option<PathBuf> {
        // 按照顺序加载配置文件
        // 1. 工作目录下的 config.toml
        // 2. 用户目录 .bcdown/config.toml
        // 3. 用户document目录 bcdown/config.toml
        let current_dir = std::env::current_dir().unwrap();
        let current_config_path = current_dir.join("config.toml");
        let home_config_path = if let Some(user_dir) = directories::UserDirs::new() {
            user_dir.home_dir().join(".bcdown/config.toml")
        } else {
            // 随便整一个不存在的路径
            current_dir.join("/114514/1919810/")
        };
        let document_config_path = if let Some(user_dir) = directories::UserDirs::new() {
            user_dir.document_dir().unwrap().join("bcdown/config.toml")
        } else {
            // 随便整一个不存在的路径
            current_dir.join("/114514/1919810/")
        };
        // 顺序检查文件是否存在
        if current_config_path.exists() {
            Some(current_config_path)
        } else if home_config_path.exists() {
            Some(home_config_path)
        } else if document_config_path.exists() {
            Some(document_config_path)
        } else {
            None
        }
    }

    pub fn load() -> Self {
        let mut log = paris::Logger::new();
        if let Some(path) = Config::path() {
            log.info(format!("加载配置文件：{}", path.display()));
            if let Ok(mut file) = std::fs::File::open(path) {
                let mut config_toml = String::new();
                file.read_to_string(&mut config_toml).unwrap();
                if let Ok(config) = toml::from_str(&config_toml) {
                    config
                } else {
                    log.error("配置文件解析失败！");
                    exit(1);
                }
            } else {
                log.error("配置文件打开失败！");
                exit(1);
            }
        } else {
            log.warn("未找到配置文件，将使用默认配置");
            Config::default()
        }
    }
    pub fn get_client(&self) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:101.0) Gecko/20100101 Firefox/101.0"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "Accept-Language",
            "zh-CN,zh;q=0.8,en-US;q=0.5,en;q=0.3".parse().unwrap(),
        );
        headers.insert(
            "Cookie",
            if self.sessdata.is_empty() {
                "".parse().unwrap()
            } else {
                format!("SESSDATA={}", self.sessdata).parse().unwrap()
            },
        );
        let client = reqwest::ClientBuilder::new().default_headers(headers);
        client.build().unwrap()
    }

    pub fn save(&self) {
        let path = if let Some(path) = Self::path() {
            // 删除原来的配置文件
            std::fs::remove_file(&path).unwrap();
            path
        } else {
            // 找一个最合适的路径
            if let Some(user_dir) = directories::UserDirs::new() {
                user_dir.document_dir().unwrap().join("bcdown/config.toml")
            } else if let Some(user_dir) = directories::UserDirs::new() {
                user_dir.home_dir().join(".bcdown/config.toml")
            } else {
                std::env::current_dir().unwrap().join("config.toml")
            }
        };
        let mut file = std::fs::File::create(path).unwrap();
        let config_toml = toml::to_string(&self).unwrap();
        file.write_all(config_toml.as_bytes()).unwrap();
    }
}
