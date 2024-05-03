use super::config::Config;
use printpdf::image_crate::EncodableLayout;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::exit;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct UserInfo {
    pub name: String,
    // uname
    pub coin: i64, // remain_gold
}

pub async fn get_user_info(config: &Config) -> Option<UserInfo> {
    let mut log = paris::Logger::new();
    let url = "https://api.bilibili.com/x/web-interface/nav";
    let wallet_url =
        "https://manga.bilibili.com/twirp/user.v1.User/GetWallet?device=pc&platform=web";
    let client = config.get_client();
    if let Ok(resp) = client.get(url).send().await {
        let value: serde_json::Value = resp.json().await.unwrap();
        let code = if let Some(code) = value.get("code") {
            if let serde_json::Value::Number(code) = code {
                code.as_i64().unwrap()
            } else {
                log.error("服务端返回的 \"code\" 字段不是数字");
                println!("调试信息：{:?}", value);
                exit(1);
            }
        } else {
            log.error("服务器返回了无法解析的数据");
            println!("调试信息：{:?}", value);
            exit(1);
        };
        if code == -101 {
            log.warn("未登录或登录已过期");
            None
        } else if code == 0 {
            let username = value
                .get("data")
                .unwrap()
                .get("uname")
                .unwrap()
                .as_str()
                .unwrap();
            // 继续查询wallet
            let resp = client.post(wallet_url).send().await.unwrap();
            let value: serde_json::Value = resp.json().await.unwrap();
            if let Some(data) = value.get("data") {
                if let Some(coin) = data.get("remain_gold") {
                    return Some(UserInfo {
                        coin: coin.as_i64().unwrap(),
                        name: username.to_string(),
                    });
                }
            }
            log.error("服务器返回了无法解析的数据");
            println!("调试信息：{:?}", value);
            exit(1);
        } else {
            log.error("服务器返回了未知错误");
            println!("调试信息：{:?}", value);
            exit(1);
        }
    } else {
        log.error("无法获取用户信息");
        exit(1);
    }
}

pub async fn get_qr_data(config: &Config) -> (String, String) {
    let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
    let mut log = paris::Logger::new();
    log.loading("加载二维码");
    let client = config.get_client();
    if let Ok(resp) = client.get(url).send().await {
        let value: serde_json::Value = resp.json().await.unwrap();
        if let Some(code) = value.get("code") {
            let code = code.as_i64().unwrap();
            if code != 0 {
                log.done();
                log.error("获取二维码失败");
                println!("调试信息：{:?}", value);
                exit(1);
            }
        } else {
            log.done();
            log.error("无法解析二维码数据");
            println!("调试信息：{:?}", value);
            exit(1);
        }
        log.done();
        let url = value
            .get("data")
            .unwrap()
            .get("url")
            .unwrap()
            .as_str()
            .unwrap();
        let qrcode_key = value
            .get("data")
            .unwrap()
            .get("qrcode_key")
            .unwrap()
            .as_str()
            .unwrap();
        (url.to_string(), qrcode_key.to_string())
    } else {
        log.done();
        log.error("无法获取二维码数据 请检查网络");
        exit(1);
    }
}

pub enum QRStatus {
    NotScan,
    // 未扫描
    Scanning,
    // 扫描中 未确认
    Complete(String),
    // 扫描完成 返回SESSDATA
    Invalid, // 无效
}

pub async fn check_qr_status(config: &Config, qrcode_key: String) -> QRStatus {
    let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
    let mut log = paris::Logger::new();
    let client = config.get_client();
    let url_qrcode = format!("{}?{}{}", url, "qrcode_key=", &qrcode_key);
    if let Ok(resp) = client.get(url_qrcode).send().await {
        let value: serde_json::Value = resp.json().await.unwrap();
        let data = value.get("data").unwrap();
        let code = data.get("code").unwrap().as_i64().unwrap();

        if code == 86101 {
            return QRStatus::NotScan;
        }
        else if code == 86090 {
            return QRStatus::Scanning;
        }
        else if code == 86038 {
            return QRStatus::Invalid;
        }
        else if code == 0 {
            let data = value.get("data").unwrap();
            // dbg!(resp.cookies());
            let url = data.get("url").unwrap().as_str().unwrap();
            let sessdata = url.split("SESSDATA=").collect::<Vec<&str>>()[1]
                .split('&')
                .collect::<Vec<&str>>()[0]
                .to_string();
            return QRStatus::Complete(sessdata);
        }
        log.error("无法获取二维码状态");
        println!("调试信息：{:?}", value);
        exit(1);
    } else {
        log.error("无法获取二维码状态 请检查网络");
        exit(1);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EpisodeInfo {
    pub title: String,
    pub id: u32,
    pub is_locked: bool,
    pub is_in_free: bool,
    pub ord: f64,
}

impl crate::lib::HasOrd for EpisodeInfo {
    fn ord(&self) -> f64 {
        self.ord
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComicInfo {
    pub id: u32,
    pub title: String,
    pub author_name: Vec<String>,
    pub styles: Vec<String>,
    pub ep_list: Vec<EpisodeInfo>,
    pub vertical_cover: String,
}

fn fix_episode_title(title: &str) -> String {
    // 将所有不能作为文件名的字符替换为下划线，包括正斜杠和反斜杠
    let mut result = title.to_string();
    for c in title.chars() {
        if c == '/'
            || c == '\\'
            || c == ':'
            || c == '*'
            || c == '?'
            || c == '"'
            || c == '<'
            || c == '>'
            || c == '|'
        {
            result = result.replace(c, "_");
        }
    }
    result
}

pub async fn get_comic_info(config: &Config, comic_id: u32) -> ComicInfo {
    let mut log = paris::Logger::new();
    log.loading("获取漫画信息...");
    let mut payload = HashMap::new();
    payload.insert("comic_id", comic_id);
    let client = config.get_client();
    let url = "https://manga.bilibili.com/twirp/comic.v1.Comic/ComicDetail?device=pc&platform=web";

    if let Ok(resp) = client.post(url).json(&payload).send().await {
        log.done();

        let value: serde_json::Value = resp.json().await.unwrap();
        let data = value.get("data").unwrap();
        match serde_json::from_value::<ComicInfo>(data.to_owned()) {
            Ok(mut value) => {
                // 如果有标题为空的episode，则使用ord作为标题
                for ep in value.ep_list.iter_mut() {
                    // 去除空字符
                    ep.title = ep.title.trim().to_string();
                    if ep.title.is_empty() {
                        ep.title = format!("第{}话", ep.ord);
                    }
                    ep.title = fix_episode_title(&ep.title);
                    // 如果 is_in_free 为 true 则设置 is_locked 为 false
                    if ep.is_in_free {
                        ep.is_locked = false;
                    }
                }
                value
            }
            Err(e) => {
                log.error("无法解析服务器响应漫画信息");
                println!("调试信息：{:?}", e);
                exit(1);
            }
        }
    } else {
        log.done();
        log.error("无法获取漫画信息 请检查网络");
        exit(1);
    }
}

pub struct EpisodeImages {
    pub host: String,
    pub paths: Vec<String>,
}

#[derive(Deserialize)]
struct ImageIndexImage {
    path: String,
}

#[derive(Deserialize)]
struct ImageIndex {
    images: Vec<ImageIndexImage>,
    host: String,
}

pub async fn get_episode_images(config: &Config, ep_id: u32) -> Option<EpisodeImages> {
    let client = config.get_client();
    let mut payload = HashMap::new();
    payload.insert("ep_id", ep_id);
    let url =
        "https://manga.bilibili.com/twirp/comic.v1.Comic/GetImageIndex?device=pc&platform=web";

    let resp = client.post(url).json(&payload).send().await.ok()?;
    let value: serde_json::Value = resp.json().await.unwrap();
    let data = value.get("data").unwrap();
    let index: ImageIndex = serde_json::from_value(data.to_owned()).ok()?;
    Some(EpisodeImages {
        host: index.host,
        paths: index.images.iter().map(|x| x.path.clone()).collect(),
    })
}

pub async fn get_image_tokens(config: &Config, paths: Vec<String>) -> Option<Vec<String>> {
    let client = config.get_client();
    let mut payload = HashMap::new();
    let paths: Vec<String> = paths.iter().map(|x| format!("\"{}\"", x)).collect();
    payload.insert("urls", format!("[{}]", paths.join(",")));
    let url = "https://manga.bilibili.com/twirp/comic.v1.Comic/ImageToken?device=pc&platform=web";
    let resp = client.post(url).json(&payload).send().await.ok()?;
    let value: serde_json::Value = resp.json().await.unwrap();
    let mut urls = Vec::new();
    let data = value.get("data").unwrap().as_array()?;
    for obj in data {
        let token = obj.get("token").unwrap().as_str()?;
        let url = obj.get("url")?.as_str()?;
        urls.push(format!("{}?token={}", url, token));
    }
    Some(urls)
}

pub async fn down_to<T: AsRef<Path>>(config: &Config, url: String, path: T) -> Option<usize> {
    // if path.as_ref().is_file() {
    //     panic!("重复下载文件: {}", path.as_ref().display());
    // }
    let client = config.get_client();
    let resp = client.get(url).send().await.ok()?; // 这里出错是在计划内的，不会强制退出

    let header_md5 = resp.headers().get("content-md5").cloned();
    let bytes = resp.bytes().await.ok()?; // 出现问题也很罕见，有时候会EOF

    if let Some(md5) = header_md5 {
        let md5 = md5.to_str().unwrap().parse::<String>().unwrap();

        // 这里的md5是base64编码的 编码的是md5的二进制数组
        let hash = md5::compute(&bytes);
        let hash = base64::encode(hash.as_bytes());
        if hash != md5 {
            // 不匹配的md5，说明文件被修改过，重新下载
            // dbg!(hash, md5);
            return None;
        }
    }
    let mut file = File::create(&path).await.ok()?;
    file.write_all(&bytes).await.unwrap();
    Some(bytes.len())
}
