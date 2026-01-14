use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Result, Context};
use ffmpeg_next as ffmpeg;
use tokio::fs;
use tokio::task;

pub struct FFmpegProcessor {
    ffmpeg_path: PathBuf,
}

impl FFmpegProcessor {
    pub fn new() -> Result<Self> {
        // Try to find FFmpeg in system or bundled location
        let ffmpeg_path = if cfg!(target_os = "windows") {
            PathBuf::from("ffmpeg.exe")
        } else {
            PathBuf::from("ffmpeg")
        };

        // Test if FFmpeg is available
        let test = Command::new(&ffmpeg_path)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if test.is_err() || !test.unwrap().success() {
            anyhow::bail!("FFmpeg not found. Please install FFmpeg and ensure it's in PATH");
        }

        Ok(Self { ffmpeg_path })
    }

    pub async fn extract_thumbnail(
        &self,
        video_path: &Path,
        timestamp: f64,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let output_path = format!("./temp/thumb_{}.png", uuid::Uuid::new_v4());
        
        task::spawn_blocking({
            let video_path = video_path.to_path_buf();
            let ffmpeg_path = self.ffmpeg_path.clone();
            
            move || {
                let output = Command::new(&ffmpeg_path)
                    .args(&[
                        "-ss",
                        &timestamp.to_string(),
                        "-i",
                        video_path.to_str().unwrap(),
                        "-vf",
                        &format!("scale={}:{},fps=1", width, height),
                    "-frames:v", "1",
                        "-f",
                        "image2pipe",
                        "-vcodec",
                        "png",
                        "-",
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                    .context("Failed to execute FFmpeg")?;

                if !output.status.success() {
                    anyhow::bail!("FFmpeg failed to extract thumbnail");
                }

                Ok(output.stdout)
            }
        })
        .await
        .context("Thumbnail extraction task failed")?
    }

    pub async fn get_video_duration(&self, video_path: &Path) -> Result<f64> {
        let duration = task::spawn_blocking({
            let video_path = video_path.to_path_buf();
            let ffmpeg_path = self.ffmpeg_path.clone();
            
            move || {
                let output = Command::new(&ffmpeg_path)
                    .args(&[
                        "-i",
                        video_path.to_str().unwrap(),
                        "-f",
                        "null",
                        "-",
                    ])
                    .stderr(Stdio::piped())
                    .stdout(Stdio::null())
                    .output()
                    .context("Failed to execute FFmpeg")?;

                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // Parse duration from FFmpeg output
                if let Some(duration_line) = stderr.lines().find(|line| line.contains("Duration:")) {
                    if let Some(duration_str) = duration_line.split("Duration: ").nth(1) {
                        if let Some(duration_part) = duration_str.split(',').next() {
                            let parts: Vec<&str> = duration_part.split(':').collect();
                            if parts.len() == 3 {
                                let hours: f64 = parts[0].parse().unwrap_or(0.0);
                                let minutes: f64 = parts[1].parse().unwrap_or(0.0);
                                let seconds: f64 = parts[2].parse().unwrap_or(0.0);
                                return Ok(hours * 3600.0 + minutes * 60.0 + seconds);
                            }
                        }
                    }
                }
                
                anyhow::bail!("Could not parse video duration");
            }
        })
        .await
        .context("Duration extraction task failed")??;

        Ok(duration)
    }

    pub async fn render_preview_frame(
        &self,
        timeline_clips: &[(PathBuf, f64, f64)], // (path, start_time_in_timeline, duration)
        current_time: f64,
        output_width: u32,
        output_height: u32,
    ) -> Result<Vec<u8>> {
        // Create a complex filter to overlay clips
        let filter_complex = self.build_filter_complex(timeline_clips, current_time, output_width, output_height);
        
        task::spawn_blocking({
            let ffmpeg_path = self.ffmpeg_path.clone();
            let clips = timeline_clips.to_vec();
            
            move || {
                let mut cmd = Command::new(&ffmpeg_path);
                
                // Add input files
                for (clip_path, _, _) in &clips {
                    cmd.arg("-i").arg(clip_path);
                }
                
                cmd.args(&[
                    "-filter_complex",
                    &filter_complex,
                    "-frames:v", "1",
                    "-f", "image2pipe",
                    "-vcodec", "png",
                    "-",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

                let output = cmd.output().context("Failed to execute FFmpeg")?;

                if !output.status.success() {
                    anyhow::bail!("FFmpeg failed to render preview");
                }

                Ok(output.stdout)
            }
        })
        .await
        .context("Preview rendering task failed")?
    }

    fn build_filter_complex(
        &self,
        clips: &[(PathBuf, f64, f64)],
        current_time: f64,
        width: u32,
        height: u32,
    ) -> String {
        let mut filters = Vec::new();
        let mut overlays = Vec::new();
        
        for (i, (_, start_time, duration)) in clips.iter().enumerate() {
            if current_time >= *start_time && current_time <= *start_time + *duration {
                let time_in_clip = current_time - start_time;
                
                // Add scaling and positioning (simplified - would need transform data)
                filters.push(format!(
                    "[{}:v]setpts=PTS-STARTPTS,scale={}:{}[v{}]",
                    i, width, height, i
                ));
                
                overlays.push(format!("[v{}]", i));
            }
        }
        
        // Combine filters
        if overlays.is_empty() {
            format!("color=c=black:size={}x{}[out]", width, height)
        } else {
            let filter_str = filters.join(";");
            let overlay_str = overlays.join("");
            format!("{};{}overlay=shortest=1[out]", filter_str, overlay_str)
        }
    }

    pub async fn export_video(
        &self,
        timeline_clips: &[(PathBuf, f64, f64, f32, Transform)], // (path, start, duration, volume, transform)
        output_path: &Path,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        
        // Create concatenation file for FFmpeg
        let concat_file = temp_dir.path().join("concat.txt");
        let mut concat_content = String::new();
        
        for (i, (clip_path, start_time, duration, volume, transform)) in timeline_clips.iter().enumerate() {
            // Generate processed clip with transformations
            let processed_clip = temp_dir.path().join(format!("clip_{}.mp4", i));
            
            self.apply_transform(clip_path, &processed_clip, *transform, *volume).await?;
            
            concat_content.push_str(&format!("file '{}'\n", processed_clip.display()));
            concat_content.push_str(&format!("duration {}\n", duration));
        }
        
        fs::write(&concat_file, concat_content).await?;
        
        task::spawn_blocking({
            let ffmpeg_path = self.ffmpeg_path.clone();
            let output_path = output_path.to_path_buf();
            
            move || {
                let output = Command::new(&ffmpeg_path)
                    .args(&[
                        "-f", "concat",
                        "-safe", "0",
                        "-i", concat_file.to_str().unwrap(),
                        "-c:v", "libx264",
                        "-preset", "medium",
                        "-crf", "23",
                        "-c:a", "aac",
                        "-b:a", "192k",
                        "-vf", &format!("scale={}:{}", width, height),
                        output_path.to_str().unwrap(),
                        "-y",
                    ])
                    .output()
                    .context("Failed to execute FFmpeg for export")?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("FFmpeg export failed: {}", stderr);
                }

                Ok(())
            }
        })
        .await
        .context("Export task failed")??;

        Ok(())
    }

    async fn apply_transform(
        &self,
        input_path: &Path,
        output_path: &Path,
        transform: Transform,
        volume: f32,
    ) -> Result<()> {
        let mut filter = String::new();
        
        // Apply crop
        if transform.crop.left > 0.0 || transform.crop.right > 0.0 || 
           transform.crop.top > 0.0 || transform.crop.bottom > 0.0 {
            filter.push_str(&format!(
                "crop=iw-{}-{}:ih-{}-{}:{}:{},",
                (transform.crop.left * 100.0) as u32,
                (transform.crop.right * 100.0) as u32,
                (transform.crop.top * 100.0) as u32,
                (transform.crop.bottom * 100.0) as u32,
                (transform.crop.left * 100.0) as u32,
                (transform.crop.top * 100.0) as u32
            ));
        }
        
        // Apply scale
        if (transform.scale - 1.0).abs() > 0.01 {
            filter.push_str(&format!("scale=iw*{}:ih*{},", transform.scale, transform.scale));
        }
        
        // Apply rotation
        if transform.rotation != 0.0 {
            filter.push_str(&format!("rotate={}*PI/180,", transform.rotation));
        }
        
        // Remove trailing comma
        if filter.ends_with(',') {
            filter.pop();
        }
        
        task::spawn_blocking({
            let ffmpeg_path = self.ffmpeg_path.clone();
            let input_path = input_path.to_path_buf();
            let output_path = output_path.to_path_buf();
            let filter = filter.clone();
            
            move || {
                let mut cmd = Command::new(&ffmpeg_path);
                cmd.arg("-i").arg(&input_path);
                
                if !filter.is_empty() {
                    cmd.args(&["-vf", &filter]);
                }
                
                // Apply audio volume
                if (volume - 1.0).abs() > 0.01 {
                    cmd.args(&["-af", &format!("volume={}", volume)]);
                }
                
                cmd.arg(&output_path)
                    .arg("-y")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                
                let status = cmd.status().context("Failed to apply transform")?;
                
                if !status.success() {
                    anyhow::bail!("FFmpeg transform failed");
                }
                
                Ok(())
            }
        })
        .await
        .context("Transform task failed")??;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub scale: f32,
    pub rotation: f32,
    pub crop: CropRect,
}

#[derive(Clone, Debug)]
pub struct CropRect {
    pub left: f32,   // 0.0 - 1.0
    pub right: f32,  // 0.0 - 1.0
    pub top: f32,    // 0.0 - 1.0
    pub bottom: f32, // 0.0 - 1.0
}