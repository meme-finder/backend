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
    images: converter::ImagesVersions,
) -> Result<(), Box<dyn Error>> {
    let id = uuid.to_string();
    let path = format!("{}/{}", &id[..2], &id[2..4]);
    let name = &id[4..];
    let base = env::var("IMAGES_DIR").unwrap_or_else(|_| String::from("./storage/images"));

    async_create_dir_all(format!("{base}/full/{path}")).await?;
    write_to_file(format!("{base}/full/{path}/{name}.webp"), images.full.webp).await?;
    write_to_file(format!("{base}/full/{path}/{name}.jpeg"), images.full.jpeg).await?;
    write_to_file(format!("{base}/full/{path}/{name}.png"), images.full.png).await?;

    async_create_dir_all(format!("{base}/normal/{path}")).await?;
    write_to_file(
        format!("{base}/normal/{path}/{name}.webp"),
        images.normal.webp,
    )
    .await?;
    write_to_file(
        format!("{base}/normal/{path}/{name}.jpeg"),
        images.normal.jpeg,
    )
    .await?;
    write_to_file(
        format!("{base}/normal/{path}/{name}.png"),
        images.normal.png,
    )
    .await?;

    async_create_dir_all(format!("{base}/preview/{path}")).await?;
    write_to_file(
        format!("{base}/preview/{path}/{name}.webp"),
        images.preview.webp,
    )
    .await?;
    write_to_file(
        format!("{base}/preview/{path}/{name}.jpeg"),
        images.preview.jpeg,
    )
    .await?;
    write_to_file(
        format!("{base}/preview/{path}/{name}.png"),
        images.preview.png,
    )
    .await?;

    async_create_dir_all(format!("{base}/original/{path}")).await?;
    write_to_file(format!("{base}/original/{path}/{name}"), images.original).await?;
    Ok(())
}
