use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::{Result, Context};
use tokio::fs;
use tokio::task;

use crate::state::{Timeline, Clip, Asset};
use super::ffmpeg::{FFmpegProcessor, Transform, CropRect};

pub struct ExportEngine {
    processor: FFmpegProcessor,
    progress_callback: Arc<RwLock<Option<Box<dyn Fn(f32) + Send + Sync>>>>,
}

impl ExportEngine {
    pub fn new() -> Result<Self> {
        let processor = FFmpegProcessor::new()?;
        
        Ok(Self {
            processor,
            progress_callback: Arc::new(RwLock::new(None)),
        })
    }
    
    pub fn set_progress_callback<F>(&self, callback: F)
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        *self.progress_callback.write() = Some(Box::new(callback));
    }
    
    pub async fn export_timeline(
        &self,
        timeline: &Timeline,
        assets: &[(Uuid, Asset)],
        output_path: &Path,
        settings: &ExportSettings,
    ) -> Result<()> {
        // Prepare clips for export
        let mut export_clips = Vec::new();
        
        for track in &timeline.tracks {
            if !track.visible || track.locked {
                continue;
            }
            
            for clip in &track.clips {
                if !clip.visible {
                    continue;
                }
                
                if let Some((_, asset)) = assets.iter().find(|(id, _)| id == &clip.asset_id) {
                    let transform = Transform {
                        scale: clip.transform.scale,
                        rotation: clip.transform.rotation,
                        crop: CropRect {
                            left: clip.transform.crop.left,
                            right: clip.transform.crop.right,
                            top: clip.transform.crop.top,
                            bottom: clip.transform.crop.bottom,
                        },
                    };
                    
                    export_clips.push((
                        asset.path.clone(),
                        clip.start_time,
                        clip.duration,
                        clip.volume * track.volume,
                        transform,
                    ));
                }
            }
        }
        
        // Sort by start time
        export_clips.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Update progress
        if let Some(callback) = &*self.progress_callback.read() {
            callback(0.1);
        }
        
        // Create temporary directory
        let temp_dir = tempfile::tempdir()?;
        
        // Process each clip
        let total_clips = export_clips.len();
        for (i, (clip_path, start_time, duration, volume, transform)) in export_clips.iter().enumerate() {
            let processed_clip = temp_dir.path().join(format!("clip_{}.mp4", i));
            
            self.processor.apply_transform(
                clip_path,
                &processed_clip,
                transform.clone(),
                *volume,
            ).await?;
            
            // Update progress
            let progress = 0.1 + 0.6 * (i as f32 / total_clips as f32);
            if let Some(callback) = &*self.progress_callback.read() {
                callback(progress);
            }
        }
        
        // Create concatenation file
        let concat_file = temp_dir.path().join("concat.txt");
        let mut concat_content = String::new();
        
        for i in 0..total_clips {
            let clip_path = temp_dir.path().join(format!("clip_{}.mp4", i));
            concat_content.push_str(&format!("file '{}'\n", clip_path.display()));
        }
        
        fs::write(&concat_file, concat_content).await?;
        
        // Update progress
        if let Some(callback) = &*self.progress_callback.read() {
            callback(0.8);
        }
        
        // Final export
        self.processor.export_video(
            &export_clips.iter().map(|(path, start, dur, vol, trans)| {
                (path.clone(), *start, *dur, *vol, trans.clone())
            }).collect::<Vec<_>>(),
            output_path,
            settings.width,
            settings.height,
            settings.fps,
        ).await?;
        
        // Update progress
        if let Some(callback) = &*self.progress_callback.read() {
            callback(1.0);
        }
        
        // Cleanup
        drop(temp_dir);
        
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ExportSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub format: String,
    pub quality: u32,
    pub audio_bitrate: String,
    pub video_preset: String,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            format: "mp4".to_string(),
            quality: 23,
            audio_bitrate: "192k".to_string(),
            video_preset: "medium".to_string(),
        }
    }
}