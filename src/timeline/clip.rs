use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clip {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub track_id: Uuid,
    pub start_time: f64,
    pub duration: f64,
    pub trim_start: f64,
    pub trim_end: f64,
    pub volume: f32,
    pub transform: Transform,
    pub visible: bool,
    pub locked: bool,
    pub effects: Vec<Effect>,
    pub speed: f32,
}

impl Clip {
    pub fn new(asset_id: Uuid, track_id: Uuid, start_time: f64, duration: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            track_id,
            start_time,
            duration,
            trim_start: 0.0,
            trim_end: 0.0,
            volume: 1.0,
            transform: Transform::default(),
            visible: true,
            locked: false,
            effects: Vec::new(),
            speed: 1.0,
        }
    }
    
    pub fn get_effective_duration(&self) -> f64 {
        self.duration / self.speed as f64
    }
    
    pub fn set_start_time(&mut self, time: f64) {
        self.start_time = time.max(0.0);
    }
    
    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration.max(0.1); // Minimum 0.1 seconds
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 2.0);
    }
    
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.1, 10.0); // 0.1x to 10x speed
    }
    
    pub fn trim_left(&mut self, amount: f64) {
        let max_trim = self.duration - 0.1; // Keep at least 0.1s
        self.trim_start = (self.trim_start + amount).clamp(0.0, max_trim);
        self.duration -= amount;
        if self.duration < 0.1 {
            self.duration = 0.1;
        }
    }
    
    pub fn trim_right(&mut self, amount: f64) {
        let max_trim = self.duration - 0.1;
        self.trim_end = (self.trim_end + amount).clamp(0.0, max_trim);
        self.duration -= amount;
        if self.duration < 0.1 {
            self.duration = 0.1;
        }
    }
    
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
    
    pub fn remove_effect(&mut self, effect_id: Uuid) -> Option<Effect> {
        let index = self.effects.iter().position(|e| e.id == effect_id)?;
        Some(self.effects.remove(index))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Transform {
    pub position: (f32, f32),
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,
    pub crop: CropRect,
    pub blend_mode: BlendMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Default for CropRect {
    fn default() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effect {
    pub id: Uuid,
    pub name: String,
    pub effect_type: EffectType,
    pub enabled: bool,
    pub parameters: Vec<EffectParameter>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectType {
    BrightnessContrast,
    HueSaturation,
    Blur,
    Sharpen,
    ColorGrade,
    ChromaKey,
    Vignette,
    Glow,
    Noise,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectParameter {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
}