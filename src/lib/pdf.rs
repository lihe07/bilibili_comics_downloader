use printpdf::{Image, Mm, PdfDocument};
use std::path::PathBuf;

const H: f64 = 297.;
// 297mm
const W: f64 = 210.;

fn calc_transformation(img_w: u32, img_h: u32, dpi: f64) -> printpdf::ImageTransform {
    // 变换方法: 中心点对齐 不允许出界 保持比例
    let img_w = img_w as f64;
    let img_h = img_h as f64;

    // 将图片的宽高转换为 mm
    let img_w_mm = img_w / dpi * 25.4;
    let img_h_mm = img_h / dpi * 25.4;

    let w = W;
    let h = H;
    let scale = if img_w > img_h {
        w / img_w_mm
    } else {
        h / img_h_mm
    };
    let x = (w - img_w_mm * scale) / 2.;
    let y = (h - img_h_mm * scale) / 2.;
    printpdf::ImageTransform {
        translate_x: Some(Mm(x)),
        translate_y: Some(Mm(y)),
        rotate: None,
        scale_x: Some(scale),
        scale_y: Some(scale),
        dpi: Some(dpi),
    }
}

fn calc_best_dpi(img_w: u32, img_h: u32) -> f64 {
    let img_w = img_w as f64;
    let img_h = img_h as f64;
    let w = W;
    let h = H;

    if img_w > img_h {
        w / img_w
    } else {
        h / img_h
    }
}

pub fn from_images(
    images: Vec<PathBuf>,
    title: &str,
    bookmark: &str,
    dpi: Option<f64>,
) -> printpdf::PdfDocumentReference {
    let (doc, page, layer) = PdfDocument::new(title, Mm(W), Mm(H), "image_layer");
    doc.add_bookmark(bookmark, page);
    let mut current_layer = doc.get_page(page).get_layer(layer);
    for (i, path) in images.iter().enumerate() {
        let d_image = printpdf::image_crate::open(path).unwrap();
        let image = Image::from_dynamic_image(&d_image);
        let dpi = dpi.unwrap_or_else(|| calc_best_dpi(d_image.width(), d_image.height()));
        image.add_to_layer(
            current_layer,
            calc_transformation(d_image.width(), d_image.height(), dpi),
        );

        if i < images.len() - 1 {
            let (page, layer) = doc.add_page(Mm(W), Mm(H), "image_layer");
            current_layer = doc.get_page(page).get_layer(layer);
        } else {
            break;
        }
    }
    doc
}

pub fn append(
    doc: printpdf::PdfDocumentReference,
    images: Vec<PathBuf>,
    bookmark: &str,
    dpi: Option<f64>,
) -> printpdf::PdfDocumentReference {
    let (page, layer) = doc.add_page(Mm(W), Mm(H), "image_layer");
    doc.add_bookmark(bookmark, page);

    let mut current_layer = doc.get_page(page).get_layer(layer);
    for (i, path) in images.iter().enumerate() {
        let d_image = printpdf::image_crate::open(path).unwrap();
        let image = Image::from_dynamic_image(&d_image);
        let dpi = dpi.unwrap_or_else(|| calc_best_dpi(d_image.width(), d_image.height()));
        image.add_to_layer(
            current_layer,
            calc_transformation(d_image.width(), d_image.height(), dpi),
        );

        if i < images.len() - 1 {
            let (page, layer) = doc.add_page(Mm(W), Mm(H), "image_layer");
            current_layer = doc.get_page(page).get_layer(layer);
        } else {
            break;
        }
    }
    doc
}
