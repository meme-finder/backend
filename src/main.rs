#![warn(clippy::cargo)]
// #![warn(clippy::restriction)]
#![allow(
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions,
//     clippy::blanket_clippy_restriction_lints,
//     clippy::missing_docs_in_private_items,
//     clippy::implicit_return,
//     clippy::expect_used,
//     clippy::exhaustive_structs,
//     clippy::shadow_reuse,
//     clippy::try_err
)]

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::http::*;
use actix_web::middleware::Logger;
use actix_web::*;

use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;

use once_cell::sync::Lazy;
use std::env;
use std::error::Error;
use std::fs::create_dir_all; // TODO

// TODO: use anyhow

mod converter;
mod models;
mod storage;

static CLIENT: Lazy<Client> = Lazy::new(|| {
    let meili_url = env::var("MEILI_URL").unwrap_or_else(|_| String::from("http://localhost:7700"));
    let meili_key = env::var("MEILI_MASTER_KEY").unwrap_or_else(|_| String::from("key"));
    Client::new(meili_url, meili_key)
});

#[get("/health")]
async fn get_health() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "available" })))
}

#[get("/images")]
async fn get_images(query: web::Query<models::Query>) -> Result<impl Responder, Box<dyn Error>> {
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
    let id = uuid::Uuid::parse_str(&id.into_inner())?;
    let image = CLIENT
        .index("images")
        .get_document::<models::ImageInfo>(id)
        .await?;
    Ok(web::Json(image))
}

#[delete("/images/{id}")]
async fn delete_image(id: web::Path<String>) -> Result<impl Responder, Box<dyn Error>> {
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
async fn post_image(
    image: web::Json<models::ImageCreationRequest>,
) -> Result<web::Json<models::ImageInfo>, Box<dyn Error>> {
    let converted = converter::convert_and_resize(image.0.image).await?;

    let id = uuid::Uuid::new_v4();

    let image_info = models::ImageInfo {
        id,
        name: image.0.name,
        description: image.0.description,
        text: image.0.text,
    };

    storage::save_images(image_info.id, converted).await?;

    CLIENT
        .index("images")
        .add_documents(&[image_info.clone()], Some("id"))
        .await?;

    Ok(web::Json(image_info))
}

async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND);
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed())),
    }
}

async fn create_index() -> Result<(), Box<dyn Error>> {
    let index: Index = CLIENT
        .create_index("images", Some("id"))
        .await?
        .wait_for_completion(&CLIENT, None, None)
        .await?
        .try_make_index(&CLIENT)
        .expect("An error happened with the index creation.");

    let settings: Settings =
        Settings::new().with_searchable_attributes(["name", "description", "text"]);

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
            &env::var("CORS_ORIGIN").unwrap_or_else(|_| String::from("https://ms.averyan.ru")),
        )
        .allowed_methods(vec!["GET", "POST", "DELETE", "UPDATE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    create_dir_all(env::var("IMAGES_DIR").unwrap_or_else(|_| String::from("./storage/images")))?;

    if !CLIENT.is_healthy().await {
        return Err("Could not join the remote server".into());
    }

    if CLIENT.get_index("images").await.is_err() {
        create_index().await?
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://[::]:8080");

    HttpServer::new(|| {
        let cors = create_cors();
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(Files::new(
                "/static/images",
                env::var("IMAGES_DIR").unwrap_or_else(|_| String::from("./storage/images")),
            ))
            .service(Files::new("/static", "static").show_files_listing())
            .service(get_health)
            .service(get_images)
            .service(post_image)
            .service(get_image)
            .service(delete_image)
            .default_service(web::to(default_handler))
    })
    .bind(("::", 8080))?
    .run()
    .await?;

    Ok(())
}
