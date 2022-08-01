use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use indicatif::{MultiProgress, ProgressBar};
use crate::lib::cache::ComicCache;
use crate::lib::cache::EpisodeCache;
use crate::lib::config::Config;
use super::pdf;
use zip::{CompressionMethod, ZipWriter};
use zip::write::FileOptions;

use serde::{Deserialize, Serialize};
use serde_json::json;


#[enum_dispatch::enum_dispatch(ExportFormatEnum)]
pub trait ExportFormat {
    fn get_extension(&self) -> &'static str;
    /// 导出单话
    fn export_single<P: AsRef<Path>>(&self, comic: &ComicCache, episode: &EpisodeCache, path: P, config: &Config);
    /// 导出多话且合并
    fn export_multiple<P: AsRef<Path>>(&self, comic: &ComicCache, episodes: Vec<&EpisodeCache>, title: &str, path: P, config: &Config, bar: &ProgressBar);
}

#[enum_dispatch::enum_dispatch]
pub enum ExportFormatEnum {
    Epub,
    PDF,
    Zip,
    Cbz,
}

pub struct PDF;


impl ExportFormat for PDF {
    fn get_extension(&self) -> &'static str {
        "pdf"
    }

    fn export_single<P: AsRef<Path>>(&self, _: &ComicCache, episode: &EpisodeCache, path: P, config: &Config) {
        let paths = episode.get_paths();
        let doc = pdf::from_images(paths, &episode.title, &episode.title, config.dpi.clone());
        let file = File::create(path.as_ref()).unwrap();
        let mut buf = BufWriter::new(file);
        doc.save(&mut buf).unwrap();
    }

    fn export_multiple<P: AsRef<Path>>(&self, _: &ComicCache, episodes: Vec<&EpisodeCache>, title: &str, path: P, config: &Config, bar: &ProgressBar) {
        let mut pdf = None;
        for (i, episode) in episodes.iter().enumerate() {
            let paths = episode.get_paths();
            if i == 0 {
                pdf = Some(pdf::from_images(paths, title, &episode.title, config.dpi.clone()));
            } else {
                pdf = Some(pdf::append(pdf.unwrap(), paths, &episode.title, config.dpi.clone()));
            }
            bar.inc(1);
        }
        let file = File::create(path.as_ref()).unwrap();
        let mut buf = BufWriter::new(file);
        pdf.unwrap().save(&mut buf).unwrap();
    }
}

//
// pub fn export_pdf(
//     split_episodes: bool,
//     comic_dir: PathBuf,
//     ep_list: Vec<&EpisodeCache>,
//     config: &Config,
//     bar: ProgressBar,
//     out_dir: PathBuf,
//     comic_cache: &ComicCache,
// ) {
//     let mut log = paris::Logger::new();
//
//     if split_episodes {
//         let mut files = Vec::new();
//         log.info("为每一话生成PDF文件...");
//         for ep in ep_list {
//             let ep_dir = comic_dir.join(format!("{}", ep.id));
//             let paths = ep.paths.iter().map(|link| {
//                 let file_name = link.split('/').last().unwrap();
//                 ep_dir.join(file_name)
//             }).collect::<Vec<_>>();
//
//             let doc = pdf::from_images(paths, ep.title.clone(), ep.title.clone(), config.dpi.clone());
//             let path = ep_dir.join(
//                 if let Some(dpi) = config.dpi {
//                     format!("{}-dpi.pdf", dpi)
//                 } else {
//                     "best.pdf".to_string()
//                 }
//             );
//             let mut file = File::create(&path).unwrap();
//             let mut buf_writer = BufWriter::new(&mut file);
//             doc.save(&mut buf_writer).unwrap();
//             files.push((path, format!("{}-{}.pdf", ep.ord, ep.title)));
//             bar.inc(1);
//         }
//         bar.finish();
//
//
//         // 将每一话的PDF文件分别复制到对应的文件夹中
//         for (path, target_name) in files {
//             std::fs::copy(&path, out_dir.join(target_name)).unwrap();
//         }
//     } else {
//         log.loading("生成PDF文件...");
//         let mut pdf = None;
//         for (i, ep) in ep_list.iter().enumerate() {
//             let ep_dir = comic_dir.join(format!("{}", ep.id));
//             let paths = ep.paths.iter().map(|link| {
//                 let file_name = link.split('/').last().unwrap();
//                 ep_dir.join(file_name)
//             }).collect::<Vec<_>>();
//             if i == 0 {
//                 pdf = Some(pdf::from_images(paths, comic_cache.title.clone(), ep.title.clone(), config.dpi.clone()));
//             } else {
//                 pdf = Some(pdf::append(pdf.unwrap(), paths, ep.title.clone(), config.dpi.clone()));
//             }
//         }
//         log.done();
//         log.success("生成PDF文件完成");
//
//         let path = out_dir.join("merged.pdf".to_string());
//         let mut file = File::create(&path).unwrap();
//         let mut buf_writer = BufWriter::new(&mut file);
//         pdf.unwrap().save(&mut buf_writer).unwrap();
//     }
// }


pub struct Epub {
    pub(crate) cover: Option<Vec<u8>>,
}

const CONTENT_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="https://www.idpf.org/2007/ops">
<body>
<img src="{src}" alt="{alt}" />
</body>
</html>"#;

const STYLE: &str = "body { margin: 0; padding: 0; } img { width: 100%; height: auto; }";

impl Epub {
    fn make_builder(&self, comic: &ComicCache, title: &str, ord: &Option<f64>) -> EpubBuilder<ZipLibrary> {
        let zip = ZipLibrary::new().unwrap();
        let mut builder = EpubBuilder::new(zip).unwrap();
        if let Some(cover) = self.cover.as_ref() {
            builder.add_cover_image("images/cover.jpg", cover.as_slice(), "image/jpeg").unwrap();
        }
        builder.metadata("title", title).unwrap();
        builder.metadata("lang", "zho").unwrap();
        comic.author_names.iter().for_each(|au| {builder.metadata("author", &(*au)[..]).unwrap(); ()} );
        comic.subjects.iter().for_each(|su| {builder.metadata("subject", &(*su)[..]).unwrap(); ()} );
        match ord {
            Some(ord) => { builder.metadata("description", &format!("{}/{}", comic.title, ord)).unwrap(); () }
            None => { builder.metadata("description", &format!("{}", comic.title)).unwrap(); () }
        }

        builder.stylesheet(STYLE.as_bytes()).unwrap();
        builder
    }

    fn guess_mime(path: &str) -> &str {
        if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            "image/jpeg"
        } else if path.ends_with(".png") {
            "image/png"
        } else if path.ends_with(".gif") {
            "image/gif"
        } else if path.ends_with(".svg") {
            "image/svg+xml"
        } else {
            "application/octet-stream"
        }
    }
}

impl ExportFormat for Epub {
    fn get_extension(&self) -> &'static str {
        "epub"
    }

    fn export_single<P: AsRef<Path>>(&self, comic: &ComicCache, episode: &EpisodeCache, path: P, _config: &Config) {
        let mut builder = self.make_builder(&comic, &episode.title, &Some(episode.ord));
        for (i, path) in episode.get_paths().iter().enumerate() {
            let file = File::open(path).unwrap();
            let mut buf_reader = BufReader::new(file);
            let mut buf = Vec::new();
            buf_reader.read_to_end(&mut buf).unwrap();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let mime = Self::guess_mime(&file_name);
            builder.add_resource(format!("images/{}/{}", episode.id, file_name), buf.as_slice(), mime).unwrap();

            builder.add_content(
                EpubContent::new(
                    format!("{}-{}.xhtml", episode.id, i),
                    CONTENT_TEMPLATE
                        .replace("{src}", &format!("images/{}/{}", episode.id, file_name))
                        .replace("{alt}", &file_name).as_bytes(),
                )
            ).unwrap();
        }
        let file = File::create(path).unwrap();
        let mut buf_writer = BufWriter::new(file);
        builder.generate(&mut buf_writer).unwrap();
    }

    fn export_multiple<P: AsRef<Path>>(&self, comic: &ComicCache, episodes: Vec<&EpisodeCache>, title: &str, path: P, _config: &Config, bar: &ProgressBar) {
        let mut builder = self.make_builder(&comic, title, &None);
        for ep in episodes {
            for (i, path) in ep.get_paths().iter().enumerate() {
                let file = File::open(path).unwrap();
                let mut buf_reader = BufReader::new(file);
                let mut buf = Vec::new();
                buf_reader.read_to_end(&mut buf).unwrap();
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let mime = Self::guess_mime(&file_name);
                builder.add_resource(format!("images/{}/{}", ep.id, file_name), buf.as_slice(), mime).unwrap();
                if i == 0 {
                    builder.add_content(
                        EpubContent::new(format!("{}.xhtml", ep.id), CONTENT_TEMPLATE.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", &file_name).as_bytes())
                            .title(&ep.title)
                    ).unwrap();
                } else {
                    builder.add_content(
                        EpubContent::new(format!("{}-{}.xhtml", ep.id, i), CONTENT_TEMPLATE.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", &file_name).as_bytes())
                            .level(2)
                    ).unwrap();
                }
            }
            bar.inc(1);
        }
        let file = File::create(path).unwrap();
        let mut buf_writer = BufWriter::new(file);
        builder.generate(&mut buf_writer).unwrap();
    }
}
//
// pub fn export_epub(
//     split_episodes: bool,
//     comic_dir: PathBuf,
//     ep_list: Vec<&EpisodeCache>,
//     bar: ProgressBar,
//     out_dir: PathBuf,
//     comic_cache: &ComicCache,
// ) {
//     let mut log = paris::Logger::new();
//     let cover_path = comic_dir.join("cover.jpg");
//     let cover = if cover_path.is_file() {
//         let file = File::open(&cover_path).unwrap();
//         let mut buf_reader = std::io::BufReader::new(file);
//         let mut buf = Vec::new();
//         buf_reader.read_to_end(&mut buf).unwrap();
//         Some(buf)
//     } else {
//         None
//     };
//     let content_template = r#"<?xml version="1.0" encoding="UTF-8"?>
// <html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
// <body>
// <img src="{src}" alt="{alt}" />
// </body>
// </html>"#;
//     let style = "body { margin: 0; padding: 0; } img { width: 100%; height: auto; }";
//
//
//     if split_episodes {
//         let mut epub_files = Vec::new();
//
//
//         for ep in ep_list {
//             let ep_dir = comic_dir.join(format!("{}", ep.id));
//             let zip = ZipLibrary::new().unwrap();
//             let mut builder = epub_builder::EpubBuilder::new(zip).unwrap();
//             if let Some(cover) = cover.clone() {
//                 builder.add_cover_image("images/cover.jpg", cover.as_slice(), "image/jpeg").unwrap();
//             }
//             builder.metadata("title", format!("{} - {}", &comic_cache.title, &ep.title)).unwrap();
//             builder.stylesheet(style.as_bytes()).unwrap();
//             for (i, link) in ep.paths.iter().enumerate() {
//                 let file_name = link.split('/').last().unwrap();
//                 let file_path = ep_dir.join(file_name);
//                 let file = File::open(&file_path).unwrap();
//                 let mut buf_reader = std::io::BufReader::new(file);
//                 let mut buf = Vec::new();
//                 buf_reader.read_to_end(&mut buf).unwrap();
//                 let mime = if file_name.ends_with("jpg") {
//                     "image/jpeg"
//                 } else {
//                     "image/png"
//                 };
//                 builder.add_resource(format!("images/{}/{}", ep.id, file_name), buf.as_slice(), mime).unwrap();
//                 builder.add_content(
//                     EpubContent::new(format!("{}-{}.xhtml", ep.id, i), content_template.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", file_name).as_bytes())
//                 ).unwrap();
//             }
//             let file = File::create(ep_dir.join("epub.epub")).unwrap();
//             let mut buf_writer = std::io::BufWriter::new(file);
//             builder.generate(&mut buf_writer).unwrap();
//             epub_files.push((ep_dir.join("epub.epub"), format!("{}-{}.epub", ep.ord, ep.title)));
//             bar.inc(1);
//         }
//         bar.finish();
//         for (path, target_name) in epub_files {
//             std::fs::copy(&path, out_dir.join(target_name)).unwrap();
//         }
//     } else {
//         let mut builder = epub_builder::EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
//         builder.stylesheet(style.as_bytes()).unwrap();
//         if let Some(cover) = cover.clone() {
//             builder.add_cover_image("images/cover.jpg", cover.as_slice(), "image/jpeg").unwrap();
//         }
//         builder.metadata("title", format!("{}", &comic_cache.title)).unwrap();
//         for ep in ep_list {
//             let ep_dir = comic_dir.join(format!("{}", ep.id));
//             for (i, link) in ep.paths.iter().enumerate() {
//                 let file_name = link.split('/').last().unwrap();
//                 let file_path = ep_dir.join(file_name);
//                 let file = File::open(&file_path).unwrap();
//                 let mut buf_reader = std::io::BufReader::new(file);
//                 let mut buf = Vec::new();
//                 buf_reader.read_to_end(&mut buf).unwrap();
//                 let mime = if file_name.ends_with("jpg") {
//                     "image/jpeg"
//                 } else {
//                     "image/png"
//                 };
//                 builder.add_resource(format!("images/{}/{}", ep.id, file_name), buf.as_slice(), mime).unwrap();
//
//                 if i == 0 {
//                     builder.add_content(
//                         EpubContent::new(format!("{}.xhtml", ep.id), content_template.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", link).as_bytes())
//                             .title(&ep.title)
//                     ).unwrap();
//                 } else {
//                     builder.add_content(
//                         EpubContent::new(format!("{}-{}.xhtml", ep.id, i), content_template.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", link).as_bytes())
//                             .level(2)
//                     ).unwrap();
//                 }
//             }
//             bar.inc(1);
//         }
//         bar.finish();
//
//
//         log.loading("正在生成EPUB文件...");
//         let file = File::create(out_dir.join("comic.epub")).unwrap();
//         let mut buf_writer = std::io::BufWriter::new(file);
//         builder.generate(&mut buf_writer).unwrap();
//     }
// }

pub struct Zip;

impl Zip {
    fn make_options() -> FileOptions {
        FileOptions::default()
            .compression_method(CompressionMethod::Stored)
    }

    fn write_single_episode(episode: &EpisodeCache, zip: &mut ZipWriter<BufWriter<File>>) {
        for (i, path) in episode.get_paths().iter().enumerate() {
            let file_ext = path.extension().unwrap().to_str().unwrap();
            let mut file = File::open(path).unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            zip.start_file(format!("{} - {}/{}.{}", episode.ord, episode.title, i, file_ext), Zip::make_options()).unwrap();
            zip.write_all(&buf).unwrap();
        }
    }
}

impl ExportFormat for Zip {
    fn get_extension(&self) -> &'static str {
        "zip"
    }

    fn export_single<P: AsRef<Path>>(&self, _: &ComicCache, episode: &EpisodeCache, path: P, _config: &Config) {
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        let mut zip = ZipWriter::new(writer);
        Zip::write_single_episode(episode, &mut zip);
        zip.finish().unwrap();
    }

    fn export_multiple<P: AsRef<Path>>(&self, _: &ComicCache, episodes: Vec<&EpisodeCache>, _title: &str, path: P, _config: &Config, bar: &ProgressBar) {
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        let mut zip = ZipWriter::new(writer);
        for episode in episodes {
            Zip::write_single_episode(episode, &mut zip);
            bar.inc(1);
        }
        zip.finish().unwrap();
    }
}
#[derive(Serialize, Deserialize)]
struct CbzCredit {
    person: String,
    role: String,
    //primary: bool,
}

#[derive(Serialize, Deserialize)]
struct CbzComicBookInfo {
    title: String,
    series: String,
    publisher: String,
    issue: Option<f64>,
    genre: String,
    language: String,
    credits: Vec<CbzCredit>,
    tags: Vec<String>,
}

pub struct Cbz;

impl Cbz {

    fn prepare_metadata(comic: &ComicCache, title: &str, ord: &Option<f64>) -> String {
        let credits = comic.author_names.iter().map(|au| CbzCredit {person: au.to_string(), role: "Creator".to_string()} ).collect::<Vec<_>>();
        let info = CbzComicBookInfo {
            title: title.to_string(),
            series: comic.title.to_string(),
            publisher: "哔哩哔哩漫画".to_string(),
            issue: *ord,
            genre: comic.subjects[0].clone(),
            language: "Chineses".to_string(),
            credits: credits,
            tags: comic.subjects.clone(),
        };

        let metadata = json!({
            "appID": "ComicTagger/",
            "ComicBookInfo/1.0": info,
        });

        let j = serde_json::to_string(&metadata).unwrap();
        //println!("\n{}\n", j);

        j
    }

    fn write_single_episode(&self, episode: &EpisodeCache, zip: &mut ZipWriter<BufWriter<File>>) {
        Zip::write_single_episode(episode, zip)
    }
}

impl ExportFormat for Cbz {
    fn get_extension(&self) -> &'static str {
        "cbz"
    }

    fn export_single<P: AsRef<Path>>(&self, comic: &ComicCache, episode: &EpisodeCache, path: P, _config: &Config) {
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        let mut zip = ZipWriter::new(writer);
        let metadata = Cbz::prepare_metadata(comic, &episode.title, &Some(episode.ord));
        zip.set_comment(metadata);
        self.write_single_episode(episode, &mut zip);
        zip.finish().unwrap();
    }

    fn export_multiple<P: AsRef<Path>>(&self, comic: &ComicCache, episodes: Vec<&EpisodeCache>, _title: &str, path: P, _config: &Config, bar: &ProgressBar) {
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        let mut zip = ZipWriter::new(writer);
        let metadata = Cbz::prepare_metadata(comic, &comic.title, &None);
        zip.set_comment(metadata);
        for episode in episodes {
            self.write_single_episode(episode, &mut zip);
            bar.inc(1);
        }
        zip.finish().unwrap();
    }
}


//
// pub fn export_zip(
//     split_episodes: bool,
//     comic_dir: PathBuf,
//     ep_list: Vec<&EpisodeCache>,
//     bar: ProgressBar,
//     out_dir: PathBuf,
//     comic_cache: &ComicCache,
// ) {
//     let options = FileOptions::default()
//         .compression_method(CompressionMethod::Stored)
//         .compression_level(Some(10));
//     if split_episodes {
//         for ep in ep_list {
//             let ep_dir = comic_dir.join(format!("{}", ep.id));
//             let target_zip = out_dir.join(format!("{} - {}.zip", ep.ord, ep.title));
//             let target_zip = File::create(target_zip).unwrap();
//             let writer = BufWriter::new(target_zip);
//             let mut zip = zip::ZipWriter::new(writer);
//             for (i, link) in ep.paths.iter().enumerate() {
//                 let file_name = link.split("/").last().unwrap();
//                 let file_path = ep_dir.join(file_name);
//                 let file_ext = file_path.extension().unwrap().to_string_lossy();
//                 let mut file = File::open(&file_path).unwrap();
//                 let mut buf = Vec::new();
//                 file.read_to_end(&mut buf).unwrap();
//                 zip.start_file(format!("{} - {}/{}.{}", ep.ord, ep.title, i, file_ext), options.clone()).unwrap();
//                 zip.write_all(buf.as_slice()).unwrap();
//             }
//             bar.inc(1);
//
//             zip.finish().unwrap();
//         }
//     } else {
//         let target_zip = out_dir.join("comic.zip");
//         let target_zip = File::create(target_zip).unwrap();
//         let writer = BufWriter::new(target_zip);
//         let mut zip = zip::ZipWriter::new(writer);
//         for ep in ep_list {
//             let ep_dir = comic_dir.join(format!("{}", ep.id));
//             for (i, link) in ep.paths.iter().enumerate() {
//                 let file_name = link.split("/").last().unwrap();
//                 let file_path = ep_dir.join(file_name);
//                 let mut file = File::open(&file_path).unwrap();
//                 let mut buf = Vec::new();
//                 file.read_to_end(&mut buf).unwrap();
//                 let file_ext = file_path.extension().unwrap().to_string_lossy();
//                 zip.start_file(format!("{}/{} - {}/{}.{}", comic_cache.title, ep.ord, ep.title, i, file_ext), options.clone()).unwrap();
//                 zip.write_all(buf.as_slice()).unwrap();
//             }
//             bar.inc(1);
//         }
//         zip.finish().unwrap();
//     }
// }

#[derive(Debug)]
pub enum Item<'a> {
    Single(&'a EpisodeCache),
    Group(Vec<&'a EpisodeCache>),
}

fn get_min_max_ord(episodes: &Vec<&EpisodeCache>) -> (f64, f64) {
    let mut min = 1.7976931348623157E+308f64;
    let mut max = 0.0;
    for ep in episodes {
        if ep.ord < min {
            min = ep.ord;
        }
        if ep.ord > max {
            max = ep.ord;
        }
    }
    (min, max)
}

impl Item<'_> {
    fn make_file_name(&self) -> String {
        match self {
            Item::Single(ep) => format!("{}. {}", ep.ord, ep.title),
            Item::Group(eps) => {
                let (min, max) = get_min_max_ord(eps);
                if min == max {
                    return format!("{}. {}", min, eps[0].title);
                }
                format!("{}-{}. {}-{}", min, max, eps[0].title, eps.last().unwrap().title)
            }
        }
    }
}

pub fn export(comic: &ComicCache, items: Vec<Item>, config: &Config, out_dir: &PathBuf, format: &ExportFormatEnum) {
    let m = MultiProgress::new();

    let bar_style = indicatif::ProgressStyle::default_bar()
        .template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
        )
        .progress_chars("##-");
    let overall_bar = m.add(ProgressBar::new(items.len() as u64));
    let bar = m.add(ProgressBar::new(1));
    bar.set_style(bar_style.clone());
    bar.set_message("等待线程状态...");
    overall_bar.set_style(bar_style.clone());
    bar.set_position(0);
    let m = std::thread::spawn(move || m.join().unwrap());
    for item in items {
        let file_name = format!("{}.{}", item.make_file_name(), format.get_extension());
        overall_bar.set_message(format!("导出 {}...", &file_name));
        let path = out_dir.join(file_name);

        match item {
            Item::Single(episode) => {
                bar.set_length(1);
                bar.set_position(0);
                format.export_single(comic, episode, path, &config);
                bar.inc(1);
                bar.finish();
            }
            Item::Group(episodes) => {
                bar.set_length(episodes.len() as u64);
                bar.set_position(0);
                // TODO
                format.export_multiple(comic, episodes, &comic.title, path, &config, &bar);
                bar.finish();
            }
        }
        overall_bar.inc(1);
    }
    overall_bar.finish_and_clear();
    bar.finish_and_clear();
    m.join().unwrap();
}