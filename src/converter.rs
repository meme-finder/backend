use actix_web::rt::spawn;
use actix_web::web::Bytes;
use image::imageops::Lanczos3;
use image::io::Reader as ImageReader;
use image::{DynamicImage, EncodableLayout};
use std::error::Error;
use std::io::Cursor;
use webp::Encoder as WebPEncoder;

pub struct ConvertedImages {
    pub png: Bytes,
    pub webp: Bytes,
    pub jpeg: Bytes,
}

impl ConvertedImages {
    pub fn new(png: Bytes, webp: Bytes, jpeg: Bytes) -> ConvertedImages {
        ConvertedImages { png, jpeg, webp }
    }
}

pub struct ImagesVersions {
    pub full: ConvertedImages,
    pub normal: ConvertedImages,
    pub preview: ConvertedImages,
    pub original: Bytes,
}

fn convert_image(img: &DynamicImage) -> Result<ConvertedImages, Box<dyn Error>> {
    let mut png: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut png), image::ImageOutputFormat::Png)?;

    let mut jpeg: Vec<u8> = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut jpeg),
        image::ImageOutputFormat::Jpeg(80),
    )?;

    let webp_encoder = WebPEncoder::from_image(img)?;
    let encoded_webp = webp_encoder.encode(65f32);

    let webp: Vec<u8> = encoded_webp.as_bytes().to_vec();

    let converted_images = ConvertedImages::new(png.into(), jpeg.into(), webp.into());
    Ok(converted_images)
}

pub async fn convert_and_resize(original_bytes: Bytes) -> Result<ImagesVersions, Box<dyn Error>> {
    spawn(async {
        let original_bytes = original_bytes;
        // Decode original image
        let img_full = ImageReader::new(Cursor::new(&original_bytes))
            .with_guessed_format()?
            .decode()?;

        // Create minified versions
        let mut img_normal = img_full.clone();
        let mut img_preview = img_full.clone();

        if img_full.width() > 256 || img_full.height() > 256 {
            img_preview = img_full.resize(256, 256, Lanczos3);
        }

        if img_full.width() > 1024 || img_full.height() > 1024 {
            img_normal = img_full.resize(1024, 1024, Lanczos3);
        }

        // Convert images
        let full = convert_image(&img_full)?;
        let normal = convert_image(&img_normal)?;
        let preview = convert_image(&img_preview)?;

        let images_versions = ImagesVersions {
            full,
            normal,
            preview,
            original: original_bytes,
        };
        Ok(images_versions)
    })
    .await?
}
