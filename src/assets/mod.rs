pub mod manager;

pub use manager::AssetManager;

use std::path::PathBuf;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub path: PathBuf,
    pub duration: f64,
    pub asset_type: AssetType,
    pub thumbnail: Option<Vec<u8>>,
    pub metadata: AssetMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AssetType {
    Video,
    Audio,
    Image,
    Gif,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Video => write!(f, "Video"),
            AssetType::Audio => write!(f, "Audio"),
            AssetType::Image => write!(f, "Image"),
            AssetType::Gif => write!(f, "GIF"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AssetMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f32>,
    pub audio_channels: Option<u32>,
    pub audio_sample_rate: Option<u32>,
    pub file_size: u64,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
}