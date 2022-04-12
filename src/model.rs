// Импортирование различных
// структур и пространств имён.
use meilisearch_sdk::document::Document;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Автоматическая генерация кода для кодирования
// и декодирования модели картинок
#[derive(Serialize, Deserialize, Debug)]
// Модель картинки, состоящая из:
// уникального идентификатора, названия,
// ссылки на картинку и описания
pub struct Image {
    // Если не задавать id картинки,
    // то будет сгенерирован случайный
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    name: String,
    link: String,
    #[serde(default)]
    description: String,
}

// Указание серверу Meilisearch, что для
// различия картинок нужно использовать
// уникальный идентификатор картинки
impl Document for Image {
    type UIDType = Uuid;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

// Автоматическая генерация кода
// для декодирования модели запроса для поиска
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
// Запрос может состоять из:
// описания текста, количество выводимых результатов (лимит),
// фильтра и прочего
pub struct Query {
    #[serde(rename = "q")]
    pub query: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub filter: Option<String>,
    pub crop_length: Option<usize>,
    pub matches: Option<bool>,
}
