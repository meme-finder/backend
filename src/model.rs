use serde::{Serialize, Deserialize};
use meilisearch_sdk::document::*;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Image {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    name: String,
    link: String,
    //tags: Vec<String>,
}

impl Document for Image {
    type UIDType = Uuid;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

#[derive(Deserialize)]
pub struct ImageSearch {
    pub q: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Selectors<T> {
    Some(T),
    All,
}

type AttributeToCrop = (String, Option<usize>);

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    #[serde(rename = "q")]
    pub query: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub filter: Option<String>,
    pub facets_distribution: Option<Selectors<Vec<String>>>,
    pub sort: Option<Vec<String>>,
    pub attributes_to_retrieve: Option<Selectors<Vec<String>>>,
    pub attributes_to_crop: Option<Selectors<Vec<AttributeToCrop>>>,
    pub crop_length: Option<usize>,
    pub attributes_to_highlight: Option<Selectors<Vec<String>>>,
    pub matches: Option<bool>,
}
