use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use anyhow::Result;
use tokio::fs;
use image::ImageFormat;

use crate::processing::ffmpeg::FFmpegProcessor;
use crate::state::{Asset, AssetType};

#[derive(Clone)]
pub struct AssetManager {
    assets: Arc<RwLock<Vec<Asset>>>,
    ffmpeg: Option<FFmpegProcessor>,
    cache_dir: PathBuf,
}

impl AssetManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let ffmpeg = FFmpegProcessor::new().ok();
        
        Self {
            assets: Arc::new(RwLock::new(Vec::new())),
            ffmpeg,
            cache_dir: cache_dir.as_ref().to_path_buf(),
        }
    }

    pub async fn import_file(&self, file_path: impl AsRef<Path>) -> Result<Uuid> {
        let path = file_path.as_ref().to_path_buf();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Determine asset type
        let asset_type = self.detect_asset_type(&path).await?;
        
        // Get duration for video/audio
        let duration = match asset_type {
            AssetType::Video | AssetType::Audio => {
                if let Some(ffmpeg) = &self.ffmpeg {
                    ffmpeg.get_video_duration(&path).await.unwrap_or(0.0)
                } else {
                    0.0
                }
            }
            AssetType::Image => 5.0, // Default image duration
            AssetType::Gif => 10.0,  // Default GIF duration
        };
        
        // Generate thumbnail
        let thumbnail = self.generate_thumbnail(&path, &asset_type).await.ok();
        
        let asset = Asset {
            id: Uuid::new_v4(),
            name: file_name,
            path,
            duration,
            asset_type,
            thumbnail,
        };
        
        let mut assets = self.assets.write();
        assets.push(asset.clone());
        
        Ok(asset.id)
    }

    async fn detect_asset_type(&self, path: &Path) -> Result<AssetType> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "mp4" | "avi" | "mov" | "mkv" | "webm" | "flv" | "wmv" => Ok(AssetType::Video),
            "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => Ok(AssetType::Audio),
            "gif" => Ok(AssetType::Gif),
            "jpg" | "jpeg" | "png" | "bmp" | "webp" | "tiff" => Ok(AssetType::Image),
            _ => {
                // Try to detect by file content
                if let Some(ffmpeg) = &self.ffmpeg {
                    if ffmpeg.get_video_duration(path).await.is_ok() {
                        return Ok(AssetType::Video);
                    }
                }
                Ok(AssetType::Image) // Default fallback
            }
        }
    }

    async fn generate_thumbnail(&self, path: &Path, asset_type: &AssetType) -> Result<Vec<u8>> {
        match asset_type {
            AssetType::Video | AssetType::Gif => {
                if let Some(ffmpeg) = &self.ffmpeg {
                    ffmpeg.extract_thumbnail(path, 0.0, 160, 90).await
                } else {
                    self.generate_fallback_thumbnail().await
                }
            }
            AssetType::Image => {
                let image_data = fs::read(path).await?;
                let image = image::load_from_memory(&image_data)?;
                let thumbnail = image.thumbnail(160, 90);
                
                let mut buffer = Vec::new();
                thumbnail.write_to(&mut buffer, ImageFormat::Png)?;
                
                Ok(buffer)
            }
            AssetType::Audio => {
                self.generate_audio_thumbnail().await
            }
        }
    }

    async fn generate_fallback_thumbnail(&self) -> Result<Vec<u8>> {
        // Create a simple colored thumbnail
        let width = 160;
        let height = 90;
        let mut imgbuf = image::ImageBuffer::new(width, height);
        
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            *pixel = image::Rgb([100, 100, 100]);
        }
        
        let mut buffer = Vec::new();
        let dyn_img = image::DynamicImage::ImageRgb8(imgbuf);
        dyn_img.write_to(&mut buffer, ImageFormat::Png)?;
        
        Ok(buffer)
    }

    async fn generate_audio_thumbnail(&self) -> Result<Vec<u8>> {
        // Create waveform-like thumbnail for audio
        let width = 160;
        let height = 90;
        let mut imgbuf = image::ImageBuffer::new(width, height);
        
        // Draw waveform visualization
        for x in 0..width {
            let amplitude = ((x as f32 / width as f32) * std::f32::consts::PI).sin();
            let bar_height = (amplitude * height as f32 / 2.0).abs() as u32;
            
            for y in (height/2 - bar_height)..(height/2 + bar_height) {
                imgbuf.put_pixel(x, y, image::Rgb([70, 130, 180]));
            }
        }
        
        let mut buffer = Vec::new();
        let dyn_img = image::DynamicImage::ImageRgb8(imgbuf);
        dyn_img.write_to(&mut buffer, ImageFormat::Png)?;
        
        Ok(buffer)
    }

    pub fn get_assets(&self) -> Vec<Asset> {
        self.assets.read().clone()
    }

    pub fn get_asset(&self, id: Uuid) -> Option<Asset> {
        self.assets.read().iter().find(|a| a.id == id).cloned()
    }

    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
}