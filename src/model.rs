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
