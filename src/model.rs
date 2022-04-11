use serde::{Deserialize, Serialize};
use uuid::Uuid;
use meilisearch_sdk::document::Document;

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    name: String,
    link: String,
    description: String,
}

impl Document for Image {
    type UIDType = Uuid;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    #[serde(rename = "q")]
    pub query: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub filter: Option<String>,
    pub crop_length: Option<usize>,
    pub matches: Option<bool>,
}
