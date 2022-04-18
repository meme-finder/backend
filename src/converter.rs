use image::io::Reader as ImageReader;
use image::EncodableLayout;
use std::io::Cursor;
use webp::Encoder as WebPEncoder;

pub struct ConvertedImages {
    pub png: Vec<u8>,
    pub webp: Vec<u8>,
    pub jpeg: Vec<u8>,
}

pub fn convert(original_bytes: Vec<u8>) -> ConvertedImages {
    let img = ImageReader::new(Cursor::new(original_bytes))
        .with_guessed_format()
        .expect("cringe")
        .decode()
        .expect("cringe");

    let mut png: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut png), image::ImageOutputFormat::Png)
        .expect("png");

    let mut jpeg: Vec<u8> = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut jpeg),
        image::ImageOutputFormat::Jpeg(80),
    )
    .expect("jpeg");

    let webp_encoder = WebPEncoder::from_image(&img).expect("webp encoder");
    let encoded_webp = webp_encoder.encode(65f32);

    let webp: Vec<u8> = encoded_webp.as_bytes().to_vec();

    let converted_images = ConvertedImages {
        png: png,
        jpeg: jpeg,
        webp: webp,
    };
    converted_images
}
