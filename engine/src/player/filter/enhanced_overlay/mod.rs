use serde::{Deserialize, Serialize};

pub mod effects;

use super::{Filters, Video};
use crate::player::utils::Media;

use effects::{Effect, Effector};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Overlay {
    pub file_path: String,
    #[serde(rename = "in")]
    pub time_in: f64,
    #[serde(rename = "out")]
    pub time_out: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<Vec<Effect>>,
    pub position: String,
    pub scale: String,
    pub rotation: f64,
    pub opacity: f64,
    #[serde(default = "default_false")]
    pub is_looped: bool,
}

fn default_false() -> bool {
    false
}

/// Implement more complex announcement on the playing media based on the playlist.
pub fn enhanced_overlay(chain: &mut Filters, node: &Media) {
    if let Some(enhanced_overlays) = &node.enhanced_overlay {
        if node.category != "advertisement" {
            for overlay in enhanced_overlays {
                if overlay.time_out > node.seek {
                    let overlay_path = overlay.file_path.replace('\\', "/").replace(':', "\\\\:");

                    let start_time = if overlay.time_in > node.seek {
                        overlay.time_in - node.seek
                    } else {
                        0.0
                    };
                    let end_time = overlay.time_out - node.seek;

                    let play_loop = if overlay.is_looped {
                        format!(":loop=0,setpts=N/(FRAME_RATE*TB)+{}/TB", start_time)
                    } else {
                        format!(",setpts=PTS+{}/TB", start_time,)
                    };

                    //setpts=PTS-STARTPTS+{}/TB
                    //:loop={play_loop}
                    let movie = format!(
                        "movie='{overlay_path}'{play_loop},format=rgba,scale={},rotate={}*PI/180,colorchannelmixer=aa={}",
                        overlay.scale, overlay.rotation, overlay.opacity
                    );
                    chain.add(&movie, 0, Video);

                    if let Some(effects) = &overlay.effect {
                        for effect in effects {
                            if let Some(overlay_effect) = effect.baker(start_time, end_time) {
                                chain.add(&overlay_effect, 0, Video);
                            }
                        }
                    }

                    let overlay_filter = format!(
                        "overlay={}:enable='lte(t, {:.2})'",
                        overlay.position, end_time
                    );
                    chain.add(&overlay_filter, 0, Video);

                    // let overlay_filter = format!(
                    //     "overlay={}:enable='between(t,{}, {})'",
                    //     overlay.position, start_time, end_time
                    // );
                    // let overlay_filter = format!("overlay={}", overlay.position);
                }
            }
        }
    }
}
