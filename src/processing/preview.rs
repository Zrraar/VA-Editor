use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task;

use crate::state::{Timeline, Asset};
use super::ffmpeg::FFmpegProcessor;

pub struct PreviewEngine {
    processor: FFmpegProcessor,
    frame_cache: Arc<Mutex<lru::LruCache<String, Vec<u8>>>>,
    worker_tx: mpsc::Sender<PreviewTask>,
}

struct PreviewTask {
    timeline: Timeline,
    timestamp: f64,
    width: u32,
    height: u32,
    callback: Box<dyn FnOnce(Result<Option<Vec<u8>>>) + Send>,
}

impl PreviewEngine {
    pub fn new() -> Result<Self> {
        let processor = FFmpegProcessor::new()?;
        let cache = Arc::new(Mutex::new(lru::LruCache::new(100))); // Cache 100 frames
        
        let (tx, mut rx) = mpsc::channel::<PreviewTask>(32);
        
        // Start preview worker
        let cache_clone = cache.clone();
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                let cache = cache_clone.clone();
                tokio::task::spawn_blocking(move || {
                    Self::process_preview_task(task, cache);
                });
            }
        });
        
        Ok(Self {
            processor,
            frame_cache: cache,
            worker_tx: tx,
        })
    }
    
    fn process_preview_task(task: PreviewTask, cache: Arc<Mutex<lru::LruCache<String, Vec<u8>>>>) {
        let cache_key = format!("{:.2}_{}x{}", task.timestamp, task.width, task.height);
        
        // Check cache first
        if let Some(cached) = cache.lock().get(&cache_key) {
            (task.callback)(Ok(Some(cached.clone())));
            return;
        }
        
        // Collect visible clips at this timestamp
        let visible_clips: Vec<(PathBuf, f64, f64)> = task.timeline.tracks
            .iter()
            .filter(|track| track.visible)
            .flat_map(|track| &track.clips)
            .filter(|clip| clip.visible)
            .filter(|clip| task.timestamp >= clip.start_time && task.timestamp <= clip.start_time + clip.duration)
            .map(|clip| {
                // In real implementation, you'd look up the asset path
                (PathBuf::from("placeholder.mp4"), clip.start_time, clip.duration)
            })
            .collect();
        
        // Generate preview using FFmpeg
        let result = if !visible_clips.is_empty() {
            match task.processor.render_preview_frame(
                &visible_clips,
                task.timestamp,
                task.width,
                task.height,
            ) {
                Ok(frame_data) => {
                    // Cache the result
                    cache.lock().put(cache_key, frame_data.clone());
                    Ok(Some(frame_data))
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(None)
        };
        
        (task.callback)(result);
    }
    
    pub async fn get_preview_frame(
        &self,
        timeline: &Timeline,
        timestamp: f64,
        width: u32,
        height: u32,
    ) -> Result<Option<Vec<u8>>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        let task = PreviewTask {
            timeline: timeline.clone(),
            timestamp,
            width,
            height,
            callback: Box::new(move |result| {
                let _ = tx.send(result);
            }),
        };
        
        self.worker_tx.send(task).await?;
        rx.await?
    }
    
    pub fn clear_cache(&self) {
        self.frame_cache.lock().clear();
    }
}