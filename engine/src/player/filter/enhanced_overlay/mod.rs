use serde::{Deserialize, Deserializer, Serialize};

pub mod effects;

use super::{Filters, Video};
use crate::player::utils::Media;

use effects::{Effect, Effector};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Overlay {
    #[serde(deserialize_with = "null_string")]
    pub file_path: String,
    #[serde(rename = "in")]
    pub in_cue: CuePoint,
    pub out: CuePoint,
    #[serde(deserialize_with = "null_string")]
    pub position: String,
    #[serde(deserialize_with = "null_string")]
    pub scale: String,
    pub rotation: f64,
    pub opacity: f64,
}

fn null_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CuePoint {
    pub seek: f64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<Vec<Effect>>,
}

/// Implement more complex announcement on the playing media based on the playlist.
pub fn enhanced_overlay(chain: &mut Filters, node: &mut Media) {
    if let Some(enhaneced_overlay) = &node.enhanced_overlay {
        if &node.category != "advertisement" {
            for overlay in enhaneced_overlay {
                if overlay.out.seek > node.seek {
                    let overlay_path = overlay.file_path.replace('\\', "/").replace(':', "\\\\:");

                    let movie = format!(
                        "movie='{overlay_path}':loop=0,setpts=N/(FRAME_RATE*TB),format=rgba,scale={},rotate={}*PI/180,colorchannelmixer=aa={}",
                        overlay.scale, overlay.rotation, overlay.opacity,
                    );
                    chain.add(&movie, 0, Video);

                    if let Some(effects) = &overlay.in_cue.effect {
                        for effect in effects {
                            if let Some(overlay_effect) =
                                effect.baker(overlay.in_cue.seek, node.seek)
                            {
                                chain.add(&overlay_effect, 0, Video);
                            };
                        }
                    }

                    if let Some(effects) = &overlay.out.effect {
                        for effect in effects {
                            if let Some(overlay_effect) = effect.baker(overlay.out.seek, node.seek)
                            {
                                chain.add(&overlay_effect, 0, Video);
                            };
                        }
                    }

                    let (adapted_seek_in, adabted_seek_out) = if node.seek < overlay.in_cue.seek {
                        let seek_in = overlay.in_cue.seek - node.seek;
                        let seek_out = overlay.out.seek - node.seek;
                        (seek_in, seek_out)
                    } else {
                        let seek_in = 0.0;
                        let seek_out = overlay.out.seek - node.seek;
                        (seek_in, seek_out)
                    };

                    let remainders = format!(
                        "overlay={}:enable='between(t,{},{})'",
                        overlay.position, adapted_seek_in, adabted_seek_out
                    );
                    chain.add(&remainders, 0, Video);
                }
            }
        }
    }
}
