use actix_web::rt::spawn;
use std::env;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;

use crate::converter;

async fn write_to_file(name: String, content: Vec<u8>) -> Result<(), Box<dyn Error>> {
    spawn(async {
        let mut file = File::create(name)?;
        let content = content;
        file.write_all(&content)?;
        Ok(())
    })
    .await?
}

async fn async_create_dir_all(path: String) -> Result<(), Box<dyn Error>> {
    spawn(async {
        create_dir_all(path)?;
        Ok(())
    })
    .await?
}

pub async fn save_images(
    uuid: uuid::Uuid,
    images: converter::ConvertedImages,
) -> Result<(), Box<dyn Error>> {
    let id = uuid.to_string();
    let path = format!("{}/{}", &id[..2], &id[2..4]);
    let name = &id[4..];
    let base = env::var("IMAGES_DIR").unwrap_or_else(|_| String::from("./storage/images"));

    async_create_dir_all(format!("{base}/webp/{path}")).await?;
    write_to_file(format!("{base}/webp/{path}/{name}.webp"), images.webp).await?;

    async_create_dir_all(format!("{base}/jpeg/{path}")).await?;
    write_to_file(format!("{base}/jpeg/{path}/{name}.jpeg"), images.jpeg).await?;

    async_create_dir_all(format!("{base}/png/{path}")).await?;
    write_to_file(format!("{base}/png/{path}/{name}.png"), images.png).await?;
    Ok(())
}
