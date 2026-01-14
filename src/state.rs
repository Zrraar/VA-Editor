use std::path::PathBuf;
use anyhow::Result;
use super::processing::ffmpeg::{FFmpegProcessor, Transform as FFmpegTransform};

// Add export methods to AppState
impl AppState {
    pub async fn export_project(&self, output_path: PathBuf) -> Result<()> {
        let timeline = self.timeline.borrow();
        let assets = self.assets.read();
        
        // Collect all clips with their transformations
        let mut export_clips = Vec::new();
        
        for track in &timeline.tracks {
            for clip in &track.clips {
                if let Some(asset) = assets.get(&clip.asset_id) {
                    let ffmpeg_transform = FFmpegTransform {
                        scale: clip.transform.scale,
                        rotation: clip.transform.rotation,
                        crop: crate::processing::ffmpeg::CropRect {
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
                        clip.volume,
                        ffmpeg_transform,
                    ));
                }
            }
        }
        
        // Sort by start time
        export_clips.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Export using FFmpeg
        let processor = FFmpegProcessor::new()?;
        processor.export_video(
            &export_clips,
            &output_path,
            1920,  // Output width
            1080,  // Output height
            30,    // FPS
        ).await
    }
    
    pub async fn get_preview_frame(&self, timestamp: f64) -> Result<Option<Vec<u8>>> {
        let timeline = self.timeline.borrow();
        let assets = self.assets.read();
        
        // Collect clips that are visible at this timestamp
        let mut visible_clips = Vec::new();
        
        for track in &timeline.tracks {
            if !track.visible {
                continue;
            }
            
            for clip in &track.clips {
                if clip.visible && timestamp >= clip.start_time && timestamp <= clip.start_time + clip.duration {
                    if let Some(asset) = assets.get(&clip.asset_id) {
                        visible_clips.push((
                            asset.path.clone(),
                            clip.start_time,
                            clip.duration,
                        ));
                    }
                }
            }
        }
        
        if visible_clips.is_empty() {
            return Ok(None);
        }
        
        let processor = FFmpegProcessor::new()?;
        let frame_data = processor.render_preview_frame(
            &visible_clips,
            timestamp,
            640,  // Preview width
            360,  // Preview height
        ).await?;
        
        Ok(Some(frame_data))
    }
}