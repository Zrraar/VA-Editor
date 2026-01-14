use uuid::Uuid;
use serde::{Deserialize, Serialize};

use super::clip::Clip;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
    pub track_type: usize, // 0 = video, 1 = audio
    pub clips: Vec<Clip>,
    pub volume: f32,
    pub visible: bool,
    pub locked: bool,
    pub muted: bool,
    pub solo: bool,
    pub height: f32,
}

impl Track {
    pub fn new(name: &str, track_type: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            track_type,
            clips: Vec::new(),
            volume: 1.0,
            visible: true,
            locked: false,
            muted: false,
            solo: false,
            height: 80.0,
        }
    }
    
    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
        self.sort_clips();
    }
    
    pub fn remove_clip(&mut self, clip_id: Uuid) -> Option<Clip> {
        let index = self.clips.iter().position(|c| c.id == clip_id)?;
        Some(self.clips.remove(index))
    }
    
    pub fn sort_clips(&mut self) {
        self.clips.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
    }
    
    pub fn get_clip_at_time(&self, time: f64) -> Option<&Clip> {
        self.clips.iter().find(|clip| {
            time >= clip.start_time && time <= clip.start_time + clip.duration
        })
    }
    
    pub fn get_overlapping_clips(&self, start_time: f64, duration: f64) -> Vec<&Clip> {
        self.clips.iter()
            .filter(|clip| {
                let clip_end = clip.start_time + clip.duration;
                let new_end = start_time + duration;
                !(new_end <= clip.start_time || start_time >= clip_end)
            })
            .collect()
    }
    
    pub fn is_time_available(&self, start_time: f64, duration: f64) -> bool {
        self.get_overlapping_clips(start_time, duration).is_empty()
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 2.0);
    }
    
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }
    
    pub fn toggle_solo(&mut self) {
        self.solo = !self.solo;
    }
    
    pub fn toggle_lock(&mut self) {
        self.locked = !self.locked;
    }
    
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
}