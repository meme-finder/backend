use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use uuid::Uuid;

// TODO: normal bytes decoder, not this workaround cringe
use base64::STANDARD;
base64_serde_type!(Base64Standard, STANDARD);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub text: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ImageCreationRequest {
    pub name: String,
    #[serde(with = "Base64Standard")]
    pub image: Vec<u8>,
    pub description: Option<String>,
    pub text: Option<String>,
    pub tags: Option<Vec<String>>,
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
