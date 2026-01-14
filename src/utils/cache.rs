use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use anyhow::Result;
use tokio::fs;
use lru::LruCache;

pub struct CacheManager {
    thumbnail_cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
    preview_cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
    metadata_cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir).ok();
        
        Self {
            thumbnail_cache: Arc::new(RwLock::new(LruCache::new(500))), // Cache 500 thumbnails
            preview_cache: Arc::new(RwLock::new(LruCache::new(100))),   // Cache 100 preview frames
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_dir,
        }
    }
    
    pub fn get_thumbnail(&self, asset_id: &Uuid, timestamp: f64, width: u32, height: u32) -> Option<Vec<u8>> {
        let key = format!("thumb_{}_{}x{}_{:.2}", asset_id, width, height, timestamp);
        self.thumbnail_cache.write().get(&key).cloned()
    }
    
    pub fn set_thumbnail(&self, asset_id: &Uuid, timestamp: f64, width: u32, height: u32, data: Vec<u8>) {
        let key = format!("thumb_{}_{}x{}_{:.2}", asset_id, width, height, timestamp);
        self.thumbnail_cache.write().put(key, data);
    }
    
    pub fn get_preview_frame(&self, timeline_hash: &str, timestamp: f64, width: u32, height: u32) -> Option<Vec<u8>> {
        let key = format!("preview_{}_{}x{}_{:.2}", timeline_hash, width, height, timestamp);
        self.preview_cache.write().get(&key).cloned()
    }
    
    pub fn set_preview_frame(&self, timeline_hash: &str, timestamp: f64, width: u32, height: u32, data: Vec<u8>) {
        let key = format!("preview_{}_{}x{}_{:.2}", timeline_hash, width, height, timestamp);
        self.preview_cache.write().put(key, data);
    }
    
    pub async fn cache_asset_file(&self, source_path: &Path) -> Result<PathBuf> {
        let file_name = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let cached_path = self.cache_dir.join(file_name);
        
        if !cached_path.exists() {
            fs::copy(source_path, &cached_path).await?;
        }
        
        Ok(cached_path)
    }
    
    pub async fn get_cached_file(&self, file_name: &str) -> Option<PathBuf> {
        let path = self.cache_dir.join(file_name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
    
    pub fn get_metadata<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.metadata_cache.read().get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
    
    pub fn set_metadata<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json_value = serde_json::to_value(value)?;
        self.metadata_cache.write().insert(key.to_string(), json_value);
        Ok(())
    }
    
    pub fn clear_thumbnails(&self) {
        self.thumbnail_cache.write().clear();
    }
    
    pub fn clear_previews(&self) {
        self.preview_cache.write().clear();
    }
    
    pub fn clear_all(&self) {
        self.clear_thumbnails();
        self.clear_previews();
        self.metadata_cache.write().clear();
        
        // Clear cache directory
        let _ = std::fs::remove_dir_all(&self.cache_dir);
        let _ = std::fs::create_dir_all(&self.cache_dir);
    }
    
    pub fn get_cache_size(&self) -> Result<u64> {
        let mut total_size = 0;
        
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }
        
        // Add in-memory cache sizes
        total_size += self.thumbnail_cache.read().len() as u64 * 1024; // Approximate
        total_size += self.preview_cache.read().len() as u64 * 102400; // Approximate
        
        Ok(total_size)
    }
}