// Clippy - специальный инструмент Rust, который
// позволяет проводить анализ кода на часто встречаемые ошибки.
// Здесь 
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
    clippy::shadow_reuse,
    clippy::try_err
)]

// Импортирование различных
// структур и пространств имён.
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::http::*;
use actix_web::middleware::Logger;
use actix_web::*;
use lazy_static::lazy_static;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;
use std::env;
use std::error::Error;

// Импортирование файлов с кодом,
// находящихся в папке проекта
mod model;

// Инициализация статической переменной CLIENT, которая
// нужна для управления базой данных Meilisearch
lazy_static! {
    static ref CLIENT: Client = {
        // Получение локального адреса, на котором запущен сервер Meilisearch
        let meili_url =
            env::var("MEILI_URL").unwrap_or_else(|_| String::from("http://localhost:7700"));
        // Получение секретного ключа
        let meili_key = env::var("MEILI_MASTER_KEY").unwrap_or_else(|_| String::from("key"));

        Client::new(meili_url, meili_key)
    };
}

// Сервис для получения JSON метаданных картинок по параметрам заданным в query
#[get("/images")]
async fn get_images(query: web::Query<model::Query>) -> Result<impl Responder, Box<dyn Error>> {
    let q = query.0;

    // Все метаданные хранятся в индексе images
    let index = CLIENT.index("images");
    let mut s = index.search();

    // Создание запроса для поиска метаданных
    s.query = q.query.as_deref();
    s.offset = q.offset;
    s.limit = q.limit;
    s.filter = q.filter.as_deref();
    s.crop_length = q.crop_length;
    s.matches = q.matches;

    // Запрос к базе данных
    let search = s.execute::<model::Image>().await?;

    // Получение результатов
    let images: Vec<model::Image> = search.hits.into_iter().map(|x| x.result).collect();

    // Кодирование результатов в JSON
    Ok(web::Json(images))
}

// Сервис для получения JSON метаданных картинки по её id
#[get("/images/{id}")]
async fn get_image(id: web::Path<String>) -> Result<impl Responder, Box<dyn Error>> {
    // Декодирование id
    let id = uuid::Uuid::parse_str(&id.into_inner())?;

    // Поиск метаданных по базе данных
    let image = CLIENT
        .index("images")
        .get_document::<model::Image>(id)
        .await?;
    
    // Кодирование результатов в JSON
    Ok(web::Json(image))
}

// Сервис для удаления метаданных картинки по её id
#[delete("/images/{id}")]
async fn delete_image(id: web::Path<String>) -> Result<impl Responder, Box<dyn Error>> {
    // Декодирование id
    let id = uuid::Uuid::parse_str(&id.into_inner())?;

    // Удаление метаданных из базы данных
    CLIENT
        .index("images")
        .delete_document(id)
        .await?
        .wait_for_completion(&CLIENT, None, None)
        .await?;
    
    // Сообщение об успешном удалении
    Ok(HttpResponse::Ok())
}

// Сервис для загрузки метаданных картинок в базу данных
#[post("/images")]
async fn post_image(image: web::Form<model::Image>) -> Result<impl Responder, Box<dyn Error>> {
    // Добавление метаданных в базу данных по модели картинки
    CLIENT
        .index("images")
        .add_documents(&[image.0], Some("id"))
        .await?;

    // Сообщение об успешной загрузке
    Ok(HttpResponse::Ok())
}

// Сервис, который при переходе на несуществующую страницу, показывает ошибку 404
async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        // Если запрос осуществляется из браузера
        // (пользователь перешёл на несуществующую страницу)
        Method::GET => {
            // То показать ему ошибку 404
            let file = NamedFile::open("static/404.html")?
                .set_status_code(StatusCode::NOT_FOUND);
            Ok(Either::Left(file))
        }
        // Иначе выдать ошибку о неправильном запросе 
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed())),
    }
}

// Создание индекса - выделенного места для хранения метаданных картинок
async fn create_index() -> Result<(), Box<dyn Error>> {
    // Создание индекса
    let index: Index = CLIENT
        .create_index("images", Some("id"))
        .await?
        .wait_for_completion(&CLIENT, None, None)
        .await?
        .try_make_index(&CLIENT)
        .expect("An error happened with the index creation.");

    // Создание настроек
    let settings: Settings = Settings::new().with_searchable_attributes(["name", "description"]);

    // Применение настроек к индексу
    index
        .set_settings(&settings)
        .await?
        .wait_for_completion(&CLIENT, None, None)
        .await?;
    
    // Сообщение об успешном создании
    Ok(())
}

// Создание CORS
// CORS - специального механизма,
// созданный для ограничения доступа к данным веб-сервера
fn create_cors() -> Cors {
    Cors::default()
        // Разрешить доступ с доменов:
        // localhost:3000, localhost:8080,
        // https://ms.averyan.ru
        .allowed_origin("http://localhost:3000")
        .allowed_origin("http://localhost:8080")
        .allowed_origin(
            &env::var("CORS_ORIGIN").unwrap_or_else(|_| String::from("https://ms.averyan.ru")),
        )
        // Разрешить использование GET, POST, DELETE, UPDATE запросов
        .allowed_methods(vec!["GET", "POST", "DELETE", "UPDATE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        // Максимальное время для кеширования запросов
        .max_age(3600)
}

// Главная функция программы, которая запускает веб-сервер
// и все его сервисы
#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Если сервер Meilisearch не запущен,
    // то прекратить работу программы
    if !CLIENT.is_healthy().await {
        Err("Could not join the remote server.")?
    }

    // Если индекса картинок не существует,
    // то создать его
    if CLIENT.get_index("images").await.is_err() {
        create_index().await?
    }

    // Инициализация логгера для вывода полезной информации
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://[::]:8080");

    // Создание веб-сервера
    HttpServer::new(|| {
        // Создание CORS
        let cors = create_cors();
        App::new()
            .wrap(cors)
            // Включение логгера
            .wrap(Logger::default())
            // Сервис показа файлов на веб-сервере
            .service(Files::new("/static", "static").show_files_listing())
            // Различные сервисы получения, удаления и загрузки картинок
            .service(get_images)
            .service(post_image)
            .service(get_image)
            .service(delete_image)
            // Сервис для несуществующих страниц
            .default_service(web::to(default_handler))
    })
    // Запуск на localhost:8080
    .bind(("::", 8080))?
    .run()
    .await?;

    // Успешное завершение работы программы
    Ok(())
}
