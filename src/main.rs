use std::io;
use actix_files::{Files, NamedFile};
use actix_web::{
    get, post,
    http::{header, Method, StatusCode},
    middleware, web, App, Either, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use lazy_static::lazy_static;
use meilisearch_sdk::{client::Client};
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;

mod model;

lazy_static! {
    static ref CLIENT: Client = Client::new(
        "http://localhost:7700",
        "key", // todo use env variable
    );
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

    let images = CLIENT.index("images");
    let mut s = images.search();

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

    let search = s
        .execute::<model::Image>().await.unwrap();
    
    let images: Vec<model::Image> = search.hits
        .into_iter()
        .map(|x| x.result)
        .collect();
    
    web::Json({
        images
    })
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
        let settings: Settings = Settings::new()
            .with_searchable_attributes(["name"]);
        //    .with_filterable_attributes(["created_at"]);

        index.set_settings(&settings)
            .await
            .expect("Could not join the remote server.")
            .wait_for_completion(&CLIENT, None, None)
            .await
            .expect("Could not join the remote server.");
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

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
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
