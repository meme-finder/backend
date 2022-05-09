#![warn(clippy::cargo)]
#![allow(clippy::cargo_common_metadata, clippy::multiple_crate_versions)]

use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::middleware::Logger;
use actix_web::{delete, get, http, post, put, web, App, HttpResponse, HttpServer, Responder, Either};

use futures_util::TryStreamExt;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::errors::{Error::Meilisearch, ErrorCode::IndexNotFound, MeilisearchError};
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;

use once_cell::sync::Lazy;
use std::env;
use std::error::Error;
use std::fs::create_dir_all;

// TODO: use anyhow

mod auth;
mod converter;
mod models;
mod storage;

static CLIENT: Lazy<Client> = Lazy::new(|| {
    let meili_url = env::var("MEILI_URL").unwrap_or_else(|_| String::from("http://localhost:7700"));
    let meili_key = env::var("MEILI_MASTER_KEY").unwrap_or_else(|_| String::from("key"));
    Client::new(meili_url, meili_key)
});

#[get("/health")]
async fn get_health() -> Result<impl Responder, Box<dyn Error>> {
    match CLIENT.health().await {
        Ok(_) => Ok("available"),
        Err(error) => Err(error.into()),
    }
}

#[get("/images")]
async fn search_images(query: web::Query<models::Query>) -> Result<impl Responder, Box<dyn Error>> {
    let q = query.0;

    let index = CLIENT.index("images");
    let mut s = index.search();

    s.query = q.query.as_deref();
    s.offset = q.offset;
    s.limit = q.limit;
    s.filter = q.filter.as_deref();
    s.crop_length = q.crop_length;
    s.matches = q.matches;

    let search = s.execute::<models::ImageInfo>().await?;

    let images: Vec<models::ImageInfo> = search.hits.into_iter().map(|x| x.result).collect();

    Ok(web::Json(images))
}

#[get("/images/{id}")]
async fn get_image(id: web::Path<String>) -> Result<impl Responder, Box<dyn Error>> {
    let image = CLIENT
        .index("images")
        .get_document::<models::ImageInfo>(&id.into_inner())
        .await?;
    Ok(web::Json(image))
}

#[delete("/images/{id}")]
async fn delete_image(
    id: web::Path<String>,
    _: auth::NeedAuth,
) -> Result<impl Responder, Box<dyn Error>> {
    let id = uuid::Uuid::parse_str(&id.into_inner())?;
    CLIENT
        .index("images")
        .delete_document(id)
        .await?
        .wait_for_completion(&CLIENT, None, None)
        .await?;
    Ok(HttpResponse::Ok())
}

#[post("/images")]
async fn post_image(mut payload: Multipart, _: auth::NeedAuth) -> Result<impl Responder, Box<dyn Error>> {
    // TODO: check that file is an image
    if let Some(mut field) = payload.try_next().await? {
        // let content_disposition = field.content_disposition();

        if let Some(chunk) = field.try_next().await? {
            // TODO: filesystem operations are blocking
            //f = web::block(move || ).await??;
            let converted = converter::convert_and_resize(chunk.to_vec()).await?;

            let image_info = models::ImageInfo::new();

            storage::save_images(image_info.id, converted).await?;

            CLIENT
                .index("images")
                .add_documents(&[&image_info], Some("id"))
                .await?;

            return Ok(Either::Left(web::Json(image_info)));
        }
    }

    Ok(Either::Right(HttpResponse::BadRequest()))
}

#[put("/images/{id}")]
async fn update_image(image: web::Path<models::ImageInfo>) -> Result<impl Responder, Box<dyn Error>> {
    // TODO: don't add document if it doesn't exist
    CLIENT.index("images").add_or_update(&[image.into_inner()], Some("id")).await?;
    Ok(HttpResponse::Ok())
}

async fn create_index() -> Result<(), Box<dyn Error>> {
    let index: Index = CLIENT
        .create_index("images", Some("id"))
        .await?
        .wait_for_completion(&CLIENT, None, None)
        .await?
        .try_make_index(&CLIENT)
        .expect("An error happened with the index creation.");

    let settings: Settings = Settings::new()
        .with_searchable_attributes(["name", "description", "text", "tags"])
        .with_filterable_attributes(["tags"]);

    index
        .set_settings(&settings)
        .await?
        .wait_for_completion(&CLIENT, None, None)
        .await?;
    Ok(())
}

fn create_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:3000")
        .allowed_origin("http://localhost:8080")
        .allowed_origin(
            &env::var("CORS_ORIGIN").unwrap_or_else(|_| String::from("https://memefinder.ru")),
        )
        .allowed_methods(vec!["GET", "POST", "DELETE", "UPDATE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    create_dir_all(env::var("IMAGES_DIR").unwrap_or_else(|_| String::from("./storage/images")))?;

    CLIENT.health().await?;

    match CLIENT.get_index("images").await {
        Err(Meilisearch(MeilisearchError {
            error_code: IndexNotFound,
            ..
        })) => {
            create_index().await?;
        }
        Err(error) => return Err(error.into()),
        _ => {}
    };

    log::info!("starting HTTP server at http://[::]:8080");

    HttpServer::new(|| {
        let cors = create_cors();
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(get_health)
            .service(search_images)
            .service(post_image)
            .service(get_image)
            .service(delete_image)
            .service(update_image)
            .service(auth::login)
    })
    .bind(("::", 8080))?
    .run()
    .await?;

    Ok(())
}
