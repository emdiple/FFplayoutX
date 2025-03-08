use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize};

pub trait Effector {
    fn baker(&self, effect_seek: f64, real_seek: f64) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Effect {
    Fade(FadeArgs),
}

impl Effector for Effect {
    fn baker(&self, effect_seek: f64, real_seek: f64) -> Option<String> {
        match self {
            Effect::Fade(args) => args.baker(effect_seek, real_seek),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]

pub struct FadeArgs {
    #[serde(deserialize_with = "null_string")]
    fade_variant: String, // e.g. "fade_in" or "fade_out"
    duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    alpha: Option<f64>,
}

fn null_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

impl Effector for FadeArgs {
    fn baker(&self, effect_seek: f64, real_seek: f64) -> Option<String> {
        let start = if matches!(self.fade_variant.to_lowercase().deref(), "fade_in" | "in")
            && real_seek <= effect_seek
        {
            Some(effect_seek - real_seek)
        } else if matches!(self.fade_variant.to_lowercase().deref(), "fade_out" | "out")
            && real_seek < (effect_seek - self.duration)
        {
            Some((effect_seek - self.duration) - real_seek)
        } else {
            None
        };

        let base = match start {
            Some(start) => {
                let fade_type = &self.fade_variant.trim_start_matches("fade_");
                format!("fade=t={}:st={}:d={}", fade_type, start, self.duration)
            }
            None => return None,
        };

        self.alpha
            .map(|alpha| format!("{}:alpha={}", base, alpha))
            .or(Some(base))
    }
}
