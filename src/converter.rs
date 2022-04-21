use actix_web::rt::spawn;
use image::io::Reader as ImageReader;
use image::EncodableLayout;
use std::error::Error;
use std::io::Cursor;
use webp::Encoder as WebPEncoder;

pub struct ConvertedImages {
    pub png: Vec<u8>,
    pub webp: Vec<u8>,
    pub jpeg: Vec<u8>,
}

pub async fn convert(original_bytes: Vec<u8>) -> Result<ConvertedImages, Box<dyn Error>> {
    spawn(async {
        let img = ImageReader::new(Cursor::new(original_bytes))
            .with_guessed_format()?
            .decode()?;

        let mut png: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut png), image::ImageOutputFormat::Png)?;

        let mut jpeg: Vec<u8> = Vec::new();
        img.write_to(
            &mut Cursor::new(&mut jpeg),
            image::ImageOutputFormat::Jpeg(80),
        )?;

        let webp_encoder = WebPEncoder::from_image(&img)?;
        let encoded_webp = webp_encoder.encode(65f32);

        let webp: Vec<u8> = encoded_webp.as_bytes().to_vec();

        let converted_images = ConvertedImages {
            png: png,
            jpeg: jpeg,
            webp: webp,
        };
        Ok(converted_images)
    })
    .await?
}
