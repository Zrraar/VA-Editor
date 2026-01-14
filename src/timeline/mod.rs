pub mod track;
pub mod clip;

pub use track::Track;
pub use clip::Clip;

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timeline {
    pub tracks: Vec<Track>,
    pub playhead: f64,
    pub duration: f64,
    pub zoom: f32,
    pub snap_enabled: bool,
    pub markers: Vec<Marker>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            tracks: vec![
                Track::new("Video Track 1", 0),
                Track::new("Audio Track 1", 1),
            ],
            playhead: 0.0,
            duration: 60.0,
            zoom: 1.0,
            snap_enabled: true,
            markers: Vec::new(),
        }
    }
    
    pub fn add_track(&mut self, name: String, track_type: usize) {
        self.tracks.push(Track::new(name, track_type));
    }
    
    pub fn remove_track(&mut self, track_id: Uuid) -> Option<Track> {
        let index = self.tracks.iter().position(|t| t.id == track_id)?;
        Some(self.tracks.remove(index))
    }
    
    pub fn move_track_up(&mut self, track_id: Uuid) -> bool {
        let index = self.tracks.iter().position(|t| t.id == track_id)?;
        if index > 0 {
            self.tracks.swap(index, index - 1);
            true
        } else {
            false
        }
    }
    
    pub fn move_track_down(&mut self, track_id: Uuid) -> bool {
        let index = self.tracks.iter().position(|t| t.id == track_id)?;
        if index < self.tracks.len() - 1 {
            self.tracks.swap(index, index + 1);
            true
        } else {
            false
        }
    }
    
    pub fn get_clip_at_position(&self, track_index: usize, time: f64) -> Option<&Clip> {
        self.tracks.get(track_index)?.clips.iter()
            .find(|clip| time >= clip.start_time && time <= clip.start_time + clip.duration)
    }
    
    pub fn get_clip_mut(&mut self, clip_id: Uuid) -> Option<&mut Clip> {
        for track in &mut self.tracks {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                return Some(clip);
            }
        }
        None
    }
    
    pub fn get_track_mut(&mut self, track_id: Uuid) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == track_id)
    }
    
    pub fn get_total_duration(&self) -> f64 {
        self.tracks.iter()
            .flat_map(|t| &t.clips)
            .map(|c| c.start_time + c.duration)
            .fold(0.0, f64::max)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Marker {
    pub id: Uuid,
    pub time: f64,
    pub color: String,
    pub label: Option<String>,
}