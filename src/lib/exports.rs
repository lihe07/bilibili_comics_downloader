use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use epub_builder::{EpubContent, ZipLibrary};
use indicatif::ProgressBar;
use crate::lib::cache::{ComicCache, EpisodeCache};
use crate::lib::config::Config;
use super::pdf;
use zip::CompressionMethod;
use zip::write::FileOptions;

pub fn export_pdf(
    split_episodes: bool,
    comic_dir: PathBuf,
    ep_list: Vec<&EpisodeCache>,
    config: &Config,
    bar: ProgressBar,
    out_dir: PathBuf,
    comic_cache: &ComicCache,
) {
    let mut log = paris::Logger::new();

    if split_episodes {
        let mut files = Vec::new();
        log.info("为每一话生成PDF文件...");
        for ep in ep_list {
            let ep_dir = comic_dir.join(format!("{}", ep.id));
            let paths = ep.paths.iter().map(|link| {
                let file_name = link.split('/').last().unwrap();
                ep_dir.join(file_name)
            }).collect::<Vec<_>>();

            let doc = pdf::from_images(paths, ep.title.clone(), format!("{} - {}", ep.ord, ep.title.clone()), config.dpi.clone());
            let path = ep_dir.join(
                if let Some(dpi) = config.dpi {
                    format!("{}-dpi.pdf", dpi)
                } else {
                    "best.pdf".to_string()
                }
            );
            let mut file = File::create(&path).unwrap();
            let mut buf_writer = BufWriter::new(&mut file);
            doc.save(&mut buf_writer).unwrap();
            files.push((path, format!("{}-{}.pdf", ep.ord, ep.title)));
            bar.inc(1);
        }
        bar.finish();


        // 将每一话的PDF文件分别复制到对应的文件夹中
        for (path, target_name) in files {
            std::fs::copy(&path, out_dir.join(target_name)).unwrap();
        }
    } else {
        log.loading("生成PDF文件...");
        let mut pdf = None;
        for (i, ep) in ep_list.iter().enumerate() {
            let ep_dir = comic_dir.join(format!("{}", ep.id));
            let paths = ep.paths.iter().map(|link| {
                let file_name = link.split('/').last().unwrap();
                ep_dir.join(file_name)
            }).collect::<Vec<_>>();
            if i == 0 {
                pdf = Some(pdf::from_images(paths, comic_cache.title.clone(), format!("{} - {}", ep.ord, ep.title.clone()), config.dpi.clone()));
            } else {
                pdf = Some(pdf::append(pdf.unwrap(), paths, format!("{} - {}", ep.ord, ep.title.clone()), config.dpi.clone()));
            }
        }
        log.done();
        log.success("生成PDF文件完成");

        let path = out_dir.join("merged.pdf".to_string());
        let mut file = File::create(&path).unwrap();
        let mut buf_writer = BufWriter::new(&mut file);
        pdf.unwrap().save(&mut buf_writer).unwrap();
    }
}

pub fn export_epub(
    split_episodes: bool,
    comic_dir: PathBuf,
    ep_list: Vec<&EpisodeCache>,
    bar: ProgressBar,
    out_dir: PathBuf,
    comic_cache: &ComicCache,
) {
    let mut log = paris::Logger::new();
    let cover_path = comic_dir.join("cover.jpg");
    let cover = if cover_path.is_file() {
        let file = File::open(&cover_path).unwrap();
        let mut buf_reader = std::io::BufReader::new(file);
        let mut buf = Vec::new();
        buf_reader.read_to_end(&mut buf).unwrap();
        Some(buf)
    } else {
        None
    };
    let content_template = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<body>
<img src="{src}" alt="{alt}" />
</body>
</html>"#;
    let style = "body { margin: 0; padding: 0; } img { width: 100%; height: auto; }";


    if split_episodes {
        let mut epub_files = Vec::new();


        for ep in ep_list {
            let ep_dir = comic_dir.join(format!("{}", ep.id));
            let zip = ZipLibrary::new().unwrap();
            let mut builder = epub_builder::EpubBuilder::new(zip).unwrap();
            if let Some(cover) = cover.clone() {
                builder.add_cover_image("images/cover.jpg", cover.as_slice(), "image/jpeg").unwrap();
            }
            builder.metadata("title", format!("{} {} - {}", &comic_cache.title, ep.ord, &ep.title)).unwrap();
            builder.stylesheet(style.as_bytes()).unwrap();
            for (i, link) in ep.paths.iter().enumerate() {
                let file_name = link.split('/').last().unwrap();
                let file_path = ep_dir.join(file_name);
                let file = File::open(&file_path).unwrap();
                let mut buf_reader = std::io::BufReader::new(file);
                let mut buf = Vec::new();
                buf_reader.read_to_end(&mut buf).unwrap();
                let mime = if file_name.ends_with("jpg") {
                    "image/jpeg"
                } else {
                    "image/png"
                };
                builder.add_resource(format!("images/{}/{}", ep.id, file_name), buf.as_slice(), mime).unwrap();
                builder.add_content(
                    EpubContent::new(format!("{}-{}.xhtml", ep.id, i), content_template.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", file_name).as_bytes())
                ).unwrap();
            }
            let file = File::create(ep_dir.join("epub.epub")).unwrap();
            let mut buf_writer = std::io::BufWriter::new(file);
            builder.generate(&mut buf_writer).unwrap();
            epub_files.push((ep_dir.join("epub.epub"), format!("{}-{}.epub", ep.ord, ep.title)));
            bar.inc(1);
        }
        bar.finish();
        for (path, target_name) in epub_files {
            std::fs::copy(&path, out_dir.join(target_name)).unwrap();
        }
    } else {
        let mut builder = epub_builder::EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
        builder.stylesheet(style.as_bytes()).unwrap();
        if let Some(cover) = cover.clone() {
            builder.add_cover_image("images/cover.jpg", cover.as_slice(), "image/jpeg").unwrap();
        }
        builder.metadata("title", format!("{}", &comic_cache.title)).unwrap();
        for ep in ep_list {
            let ep_dir = comic_dir.join(format!("{}", ep.id));
            for (i, link) in ep.paths.iter().enumerate() {
                let file_name = link.split('/').last().unwrap();
                let file_path = ep_dir.join(file_name);
                let file = File::open(&file_path).unwrap();
                let mut buf_reader = std::io::BufReader::new(file);
                let mut buf = Vec::new();
                buf_reader.read_to_end(&mut buf).unwrap();
                let mime = if file_name.ends_with("jpg") {
                    "image/jpeg"
                } else {
                    "image/png"
                };
                builder.add_resource(format!("images/{}/{}", ep.id, file_name), buf.as_slice(), mime).unwrap();

                if i == 0 {
                    builder.add_content(
                        EpubContent::new(format!("{}.xhtml", ep.id), content_template.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", link).as_bytes())
                            .title(format!("{} - {}", ep.ord, ep.title))
                    ).unwrap();
                } else {
                    builder.add_content(
                        EpubContent::new(format!("{}-{}.xhtml", ep.id, i), content_template.replace("{src}", &format!("./images/{}/{}", ep.id, file_name)).replace("{alt}", link).as_bytes())
                            .level(2)
                    ).unwrap();
                }
            }
            bar.inc(1);
        }
        bar.finish();


        log.loading("正在生成EPUB文件...");
        let file = File::create(out_dir.join("comic.epub")).unwrap();
        let mut buf_writer = std::io::BufWriter::new(file);
        builder.generate(&mut buf_writer).unwrap();
    }
}

pub fn export_zip(
    split_episodes: bool,
    comic_dir: PathBuf,
    ep_list: Vec<&EpisodeCache>,
    bar: ProgressBar,
    out_dir: PathBuf,
    comic_cache: &ComicCache,
) {
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Zstd)
        .compression_level(Some(10));
    if split_episodes {
        for ep in ep_list {
            let ep_dir = comic_dir.join(format!("{}", ep.id));
            let target_zip = out_dir.join(format!("{} - {}.zip", ep.ord, ep.title));
            let target_zip = File::create(target_zip).unwrap();
            let writer = BufWriter::new(target_zip);
            let mut zip = zip::ZipWriter::new(writer);
            for (i, link) in ep.paths.iter().enumerate() {
                let file_name = link.split("/").last().unwrap();
                let file_path = ep_dir.join(file_name);
                let file_ext = file_path.extension().unwrap().to_string_lossy();
                let mut file = File::open(&file_path).unwrap();
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                zip.start_file(format!("{} - {}/{}.{}", ep.ord, ep.title, i, file_ext), options.clone()).unwrap();
                zip.write_all(buf.as_slice()).unwrap();
            }
            bar.inc(1);

            zip.finish().unwrap();
        }
    } else {
        let target_zip = out_dir.join("comic.zip");
        let target_zip = File::create(target_zip).unwrap();
        let writer = BufWriter::new(target_zip);
        let mut zip = zip::ZipWriter::new(writer);
        for ep in ep_list {
            let ep_dir = comic_dir.join(format!("{}", ep.id));
            for (i, link) in ep.paths.iter().enumerate() {
                let file_name = link.split("/").last().unwrap();
                let file_path = ep_dir.join(file_name);
                let mut file = File::open(&file_path).unwrap();
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                let file_ext = file_path.extension().unwrap().to_string_lossy();
                zip.start_file(format!("{}/{} - {}/{}.{}", comic_cache.title, ep.ord, ep.title, i, file_ext), options.clone()).unwrap();
                zip.write_all(buf.as_slice()).unwrap();
            }
            bar.inc(1);
        }
        zip.finish().unwrap();
    }
}