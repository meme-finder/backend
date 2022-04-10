#![warn(clippy::cargo)]
#![warn(clippy::restriction)]
#![allow(
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions,
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::expect_used,
    clippy::exhaustive_structs,
)]

use actix_files::{Files, NamedFile};
use actix_web::*;
use actix_web::http::*;
use lazy_static::lazy_static;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;
use std::io;
use std::env;

mod model;

lazy_static! {
    static ref CLIENT: Client = {
        let meili_url = env::var("MEILI_URL").unwrap_or_else(|_| String::from("http://localhost:7700"));
        let meili_key = env::var("MEILI_MASTER_KEY").unwrap_or_else(|_| String::from("key"));
        Client::new(meili_url, meili_key)
    };
}

// fn convert_vec_string_to_str<'a>(x: Vec<String>) -> &'a [&'a str] {
//     x.iter().map(|y| y.as_str()).collect::<Vec<_>>().as_slice()
// }

// fn convert_selectors<'a>(x: model::Selectors<Vec<String>>) -> search::Selectors<&'a [&'a str]> {
//     match x {
//         model::Selectors::Some(x) => search::Selectors::Some(convert_vec_string_to_str(x)),
//         model::Selectors::All => search::Selectors::All,
//     }
// }

// fn convert<'a>(x: model::Selectors<Vec<(String, Option<usize>)>>) -> search::Selectors<&'a [(&'a str, Option<usize>)]> {
//     match x {
//         model::Selectors::Some(x) => search::Selectors::Some(x.iter().map(|y| (y.0.as_str(), y.1)).collect::<Vec<_>>().as_slice()),
//         model::Selectors::All => search::Selectors::All,
//     }
// }

#[get("/images")]
async fn get_images(query: web::Query<model::Query>) -> impl Responder {
    let q = query.0;

    let index = CLIENT.index("images");
    let mut s = index.search();

    s.query = q.query.as_deref();
    s.offset = q.offset;
    s.limit = q.limit;
    s.filter = q.filter.as_deref();
    s.crop_length = q.crop_length;
    s.matches = q.matches;
    // s.sort = q.sort.map(convert_vec_string_to_str);
    // s.facets_distribution = q.facets_distribution.map(convert_selectors);
    // s.attributes_to_retrieve = q.attributes_to_retrieve.map(convert_selectors);
    // s.attributes_to_highlight = q.attributes_to_highlight.map(convert_selectors);
    // s.attributes_to_crop = q.attributes_to_crop.map(convert);

    let search = s.execute::<model::Image>().await.unwrap();

    let images: Vec<model::Image> = search.hits.into_iter().map(|x| x.result).collect();

    web::Json(images)
}

#[post("/images")]
async fn post_images(image: web::Form<model::Image>) -> impl Responder {
    let result = CLIENT
        .index("images")
        .add_documents(&[image.0], Some("id"))
        .await;

    match result {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}

async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND);
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    if CLIENT.get_index("images").await.is_err() {
        let index: Index = CLIENT
            .create_index("images", Some("id"))
            .await
            .expect("Could not join the remote server.")
            .wait_for_completion(&CLIENT, None, None)
            .await
            .expect("Could not join the remote server.")
            .try_make_index(&CLIENT)
            .expect("An error happened with the index creation.");

        // https://docs.meilisearch.com/learn/configuration/settings.html#index-settings
        let settings: Settings = Settings::new().with_searchable_attributes(["name"]);
        //    .with_filterable_attributes(["created_at"]);

        index
            .set_settings(&settings)
            .await
            .expect("Could not join the remote server.")
            .wait_for_completion(&CLIENT, None, None)
            .await
            .expect("Could not join the remote server.");
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://[::]:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/test").to(|req: HttpRequest| match *req.method() {
                    Method::GET => HttpResponse::Ok(),
                    Method::POST => HttpResponse::MethodNotAllowed(),
                    _ => HttpResponse::NotFound(),
                }),
            )
            .service(
                web::resource("/").route(web::get().to(|req: HttpRequest| async move {
                    println!("{:?}", req);
                    HttpResponse::Found()
                        .insert_header((header::LOCATION, "static/welcome.html"))
                        .finish()
                })),
            )
            .service(Files::new("/static", "static").show_files_listing())
            .default_service(web::to(default_handler))
    })
    .bind(("::", 8080))?
    .run()
    .await
}
