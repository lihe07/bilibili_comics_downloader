use lazy_static::lazy_static;
use log::{error, info, warn};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

lazy_static! {
    pub static ref CONFIG: Config = {
        let config = Config::load();
        mkdir_or_exit(&config.cache_dir);
        mkdir_or_exit(&config.default_download_dir);
        config
    };
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub sessdata: String,
    pub cache_dir: String,
    pub default_download_dir: String,
    pub dpi: Option<f64>,
}

fn mkdir_or_exit<T: AsRef<Path>>(path: T) {
    if std::fs::create_dir_all(&path).is_err() {
        error!("无法创建目录：{}", path.as_ref().display());
        exit(1);
    }
}

impl Default for Config {
    fn default() -> Self {
        info!("初始化默认配置");

        let mut base_dir = if let Some(document_dir) = dirs::document_dir() {
            document_dir.join("bcdown")
        } else if let Some(home_dir) = directories::UserDirs::new().and_then(|d| d.home_dir()) {
            home_dir.join(".bcdown")
        } else if let Ok(cwd) = std::env::current_dir() {
            cwd.join(".bcdown")
        } else {
            error!("找不到合适的暂存位置！");
            exit(1);
        };

        let default_download_dir = if let Some(download_dir) = dirs::download_dir() {
            download_dir
        } else {
            base_dir.join("download")
        };

        let config_path = base_dir.join("config.toml");
        let cache_dir = base_dir.join("cache");

        info!("配置文件路径：{}", config_path.display());
        info!("缓存目录：{}", cache_dir.display());
        info!("默认下载目录：{}", default_download_dir.display());

        // 创建目录
        mkdir_or_exit(cache_dir.as_path());
        mkdir_or_exit(default_download_dir.as_path());

        let config = Config {
            sessdata: "".to_string(),
            cache_dir: cache_dir.to_string_lossy().to_string(),
            default_download_dir: default_download_dir.to_string_lossy().to_string(),
            dpi: None,
        };

        // 写入配置文件
        let mut config_file = std::fs::File::create(config_path).unwrap();
        let config_toml = toml::to_string(&config).unwrap();
        config_file.write_all(config_toml.as_bytes()).unwrap();

        config
    }
}

impl Config {
    /// 按照顺序寻找配置文件
    /// 1. ./config.toml
    /// 2. ~/.bcdown/config.toml
    /// 3. ~/Documents/bcdown/config.toml
    /// 如果找不到，则返回 None
    fn path() -> Option<PathBuf> {
        let current_dir = std::env::current_dir().unwrap();
        let config_path = current_dir.join("config.toml");

        if config_path.exists() {
            return Some(config_path);
        }

        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(".bcdown/config.toml");
            if config_path.exists() {
                return Some(config_path);
            }
        }

        if let Some(document_dir) = dirs::document_dir() {
            let config_path = document_dir.join("bcdown/config.toml");
            if config_path.exists() {
                return Some(config_path);
            }
        }

        None
    }

    /// 加载配置文件或初始化默认配置
    pub fn load() -> Self {
        if let Some(path) = Config::path() {
            info!("加载配置文件：{}", path.display());
            if let Ok(mut file) = std::fs::File::open(path) {
                let mut config_toml = String::new();
                file.read_to_string(&mut config_toml).unwrap();
                if let Ok(config) = toml::from_str(&config_toml) {
                    config
                } else {
                    error!("配置文件解析失败！");
                    exit(1);
                }
            } else {
                error!("配置文件打开失败！");
                exit(1);
            }
        } else {
            warn!("未找到配置文件，将使用默认配置");
            Config::default()
        }
    }

    /// 根据配置文件中的sessdata获取一个reqwest::Client
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
        let mut file = std::fs::File::create(Self::path().unwrap()).unwrap();
        let config_toml = toml::to_string(&self).unwrap();
        file.write_all(config_toml.as_bytes()).unwrap();
    }
}
