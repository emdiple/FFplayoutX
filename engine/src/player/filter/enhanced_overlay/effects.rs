use regex::Regex;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub trait Effector {
    fn baker(&self, start: f64, end: f64) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Effect {
    Fade(FadeArgs),
    AddText(AddTextArgs),
}

impl Effector for Effect {
    fn baker(&self, start: f64, end: f64) -> Option<String> {
        match self {
            Effect::Fade(args) => args.baker(start, end),
            Effect::AddText(args) => args.baker(start, end),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FadeArgs {
    fade_variant: String, // e.g. "fade_in" or "fade_out"
    duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    alpha: Option<f64>,
}

impl Effector for FadeArgs {
    fn baker(&self, start: f64, end: f64) -> Option<String> {
        let start = if matches!(self.fade_variant.to_lowercase().deref(), "fade_in" | "in") {
            Some(start)
        } else if matches!(self.fade_variant.to_lowercase().deref(), "fade_out" | "out") {
            Some(end - self.duration)
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

const TEXT_DEFAUTL_SIZE: f64 = 20.0;
const TEXT_DEFAUTL_COLOR: &str = "white";
const TEXT_DEFAUTL_TIEM_SHIFT: f64 = 0.0;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AddTextArgs {
    text: String,
    position: String, // example: x=(W-text_w)/2:y=(H-text_h)/2
    duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    alpha: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    font_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "t_shift")]
    time_shift: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    font_size: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    font_color: Option<String>,
    is_shadow: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shadow_amount: Option<String>,
    #[serde(rename = "is_fin")]
    is_fade_in: bool,
    #[serde(rename = "is_fout")]
    is_fade_out: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fade_amount: Option<f64>,
}

impl Effector for AddTextArgs {
    fn baker(&self, start: f64, _end: f64) -> Option<String> {
        let start = start + self.time_shift.unwrap_or(TEXT_DEFAUTL_TIEM_SHIFT);
        let end = start + self.duration;
        let text = &self.text;
        let position = &self.position;
        let font_size = &self.font_size.unwrap_or(TEXT_DEFAUTL_SIZE);
        let font_color = &self
            .font_color
            .clone()
            .unwrap_or(TEXT_DEFAUTL_COLOR.to_string());

        let alpha = self.alpha.unwrap_or(1.0);
        let fade_amount = self.fade_amount.unwrap_or(1.0);
        let fade_in = if self.is_fade_in {
            format!(
            "if(lt(t,{start}),0,if(lt(t,{start}+{fade_amount}),{alpha}*(t-{start})/{fade_amount},{alpha}))"
            )
        } else {
            alpha.to_string()
        };

        let fade_out = if self.is_fade_out {
            format!("if(gt(t,{end}-{fade_amount}),{alpha}*(1-(t-({end}-{fade_amount}))/{fade_amount}),{alpha})")
        } else {
            alpha.to_string()
        };

        let alpha_expr = if self.is_fade_in && self.is_fade_out {
            format!("if(lt(t,{start}),0,if(lt(t,{start}+{fade_amount}),{alpha}*(t-{start})/{fade_amount},{fade_out}))")
        } else if self.is_fade_in {
            fade_in
        } else if self.is_fade_out {
            fade_out
        } else {
            alpha.to_string()
        };

        let shadow = if self.is_shadow {
            let raw_shdw_val = self.shadow_amount.clone().unwrap_or("x=2, y=2".to_string());
            let re = Regex::new(r"x=([^:]+):y=([^:]+)").unwrap();
            let captures = re.captures(&raw_shdw_val)?;
            let shdw_x = captures.get(1)?.as_str();
            let shdw_y = captures.get(2)?.as_str();
            format!(":shadowx={shdw_x}:shadowy={shdw_y}")
        } else {
            "".to_string()
        };

        let font_path = self
            .font_path
            .clone()
            .map(|file_path| format!("fontfile={}:", file_path))
            .unwrap_or_default();

        let filter = format!(
            "drawtext=text='{text}':{position}:fontsize={font_size}:{font_path}fontcolor={font_color}{shadow}:alpha='{alpha_expr}':enable='between(t,{start},{end})'"
        );

        Some(filter)
    }
}
