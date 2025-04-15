use regex::Regex;
use serde::{Deserialize, Serialize};

// Define constants at the top for clarity and reuse
const TEXT_DEFAULT_SIZE: f64 = 20.0;
const TEXT_DEFAULT_COLOR: &str = "white";
const TEXT_DEFAULT_TIME_SHIFT: f64 = 0.0;
const TEXT_BASE_SCROLLING_SPEED: f64 = 10.0;
const TEXT_DEFAULT_SCROLL_DELAY: f64 = 5.0;

/// Trait for effects that generate FFmpeg filter strings
pub trait Effector {
    fn baker(&self, start: f64, end: f64) -> Option<String>;
}

/// Enum representing different effect types
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

/// Arguments for the Fade effect
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FadeArgs {
    fade_variant: String, // "fade_in" or "fade_out"
    duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    alpha: Option<f64>,
}

impl Effector for FadeArgs {
    fn baker(&self, start: f64, end: f64) -> Option<String> {
        let start_time = match self.fade_variant.to_lowercase().as_str() {
            "fade_in" | "in" => Some(start),
            "fade_out" | "out" => Some(end - self.duration),
            _ => None,
        };

        let base = start_time.map(|st| {
            let fade_type = self.fade_variant.trim_start_matches("fade_");
            format!("fade=t={}:st={}:d={}", fade_type, st, self.duration)
        })?;

        self.alpha
            .map(|alpha| format!("{}:alpha={}", base, alpha))
            .or(Some(base))
    }
}

/// Arguments for the AddText effect
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AddTextArgs {
    text: String,
    position: String, // e.g., "x=(W-text_w)/2:y=(H-text_h)/2"
    duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    visible_len: Option<String>,
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

impl AddTextArgs {
    /// Builds the alpha expression for fade effects
    fn build_alpha_expr(&self, start: f64, end: f64) -> String {
        let alpha = self.alpha.unwrap_or(1.0);
        let fade_amount = self.fade_amount.unwrap_or(1.0);

        let fade_in_expr = if self.is_fade_in {
            format!(
                "if(lt(t,{start}),0,if(lt(t,{start}+{fade_amount}),{alpha}*(t-{start})/{fade_amount},{alpha}))"
            )
        } else {
            alpha.to_string()
        };

        let fade_out_expr = if self.is_fade_out {
            format!(
                "if(gt(t,{end}-{fade_amount}),{alpha}*(1-(t-({end}-{fade_amount}))/{fade_amount}),{alpha})"
            )
        } else {
            alpha.to_string()
        };

        format!("min({}, {})", fade_in_expr, fade_out_expr)
    }

    /// Generates the drawtext filter string with proper text escaping
    fn text_preset(&self, text: &str, text_pos: &str, start: f64, _end: f64) -> Option<String> {
        let escaped_text = text.replace("'", "\\'"); // Escape single quotes for FFmpeg
        let start = start + self.time_shift.unwrap_or(TEXT_DEFAULT_TIME_SHIFT);
        let end = start + self.duration;
        let font_size = self.font_size.unwrap_or(TEXT_DEFAULT_SIZE);
        let font_color = self.font_color.as_deref().unwrap_or(TEXT_DEFAULT_COLOR);

        let alpha_expr = self.build_alpha_expr(start, end);

        let alpha_expr = if let Some(visible_len) = &self.visible_len {
            let (_, vis_px, vis_marg) = extract_scroll_components(visible_len);
            let (def_pos_x, _) = extract_position_components(&self.position);
            format!(
                "if(lt(x,{def_pos_x}-tw/2),if(gt(x,{def_pos_x}-{vis_marg}),{alpha}-(({alpha}/{vis_marg})*(({def_pos_x})-x)),0),if(gt(x,{def_pos_x}+{vis_px}),if(gt(x,{def_pos_x}+{vis_px}+{vis_marg}),0,({alpha}/{vis_marg})*({def_pos_x}+{vis_px}+{vis_marg}-x)),{alpha_expr}))",
                alpha = alpha_expr
            )
        } else {
            alpha_expr
        };

        let shadow = if self.is_shadow {
            let raw_shdw_val = self.shadow_amount.as_deref().unwrap_or("x=2:y=2");
            let re = Regex::new(r"x=([^:]+):y=([^:]+)").unwrap();
            if let Some(captures) = re.captures(raw_shdw_val) {
                let shdw_x = captures.get(1).map_or("2", |m| m.as_str());
                let shdw_y = captures.get(2).map_or("2", |m| m.as_str());
                format!(":shadowx={}:shadowy={}", shdw_x, shdw_y)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let font_path = self
            .font_path
            .as_deref()
            .map(|path| format!("fontfile={}:", path))
            .unwrap_or_default();

        let filter = format!(
            "drawtext=text='{}':{}:fontsize={}:{}fontcolor={}{}:alpha='{}':enable='between(t,{},{})'",
            escaped_text, text_pos, font_size, font_path, font_color, shadow, alpha_expr, start, end
        );
        Some(filter)
    }
}

impl Effector for AddTextArgs {
    fn baker(&self, start: f64, end: f64) -> Option<String> {
        let text = self.text.clone();
        if self.visible_len.is_some() {
            let uppercase_text = text.to_uppercase(); // todo: Handle this latter to accept both uppercase and lowecase chars
            let mut txt_filters = Vec::new();
            let mut current_pos_x = 0.0;

            for char in uppercase_text.chars() {
                let char_pos = {
                    let char_width =
                        get_char_width(char, self.font_size.unwrap_or(TEXT_DEFAULT_SIZE))
                            .unwrap_or(TEXT_DEFAULT_SIZE * 0.6);
                    let (pos_x, pos_y) = extract_position_components(&self.position);
                    let new_x = format!("x={}+{}", pos_x, current_pos_x);
                    let new_y = format!("y={}", pos_y);
                    let new_pos = format!("{}:{}", new_x, new_y);

                    current_pos_x += char_width * 0.9;
                    let (scroll_dir, _, _) =
                        extract_scroll_components(self.visible_len.as_ref().unwrap());
                    scrolling(
                        start,
                        &new_pos,
                        TEXT_BASE_SCROLLING_SPEED + 18.0,
                        TEXT_DEFAULT_SCROLL_DELAY,
                        scroll_dir,
                    )
                };

                let char_filter = self.text_preset(&char.to_string(), &char_pos, start, end);
                txt_filters.push(char_filter);
            }
            let txt_filter = txt_filters
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(",");
            Some(txt_filter)
        } else {
            self.text_preset(&text, &self.position, start, end)
            // todo: Handle this latter to accept both uppercase and lowecase chars
        }
    }
}

/// Extracts x and y components from a position string
fn extract_position_components(position: &str) -> (String, String) {
    let re = Regex::new(r"x=([^:]+):y=([^:]+)").expect("Invalid regex pattern");
    if let Some(captures) = re.captures(position) {
        (captures[1].to_string(), captures[2].to_string())
    } else {
        ("0".to_string(), "0".to_string())
    }
}

/// Extracts scrolling parameters from a scroll string
fn extract_scroll_components(scroll_elements: &str) -> (char, String, String) {
    let re =
        Regex::new(r"dir=([LR]):vis_px=([^:]+):margin_px=([^:]+)").expect("Invalid regex pattern");
    if let Some(captures) = re.captures(scroll_elements) {
        let direction = captures[1].chars().next().unwrap_or('L');
        let vis_val = captures[2].to_string();
        let marg_val = captures[3].to_string();
        (direction, vis_val, marg_val)
    } else {
        ('L', "0".to_string(), "0".to_string())
    }
}

/// Generates a scrolling position expression
fn scrolling(in_time: f64, pos: &str, speed: f64, delay: f64, dir: char) -> String {
    let re = Regex::new(r"x=([^:]+)").expect("Invalid regex pattern");
    if let Some(captures) = re.captures(pos) {
        let x_value = &captures[1];
        let dir_sign = match dir {
            'L' => "-",
            'R' => "+",
            _ => "-",
        };
        let scrolling_expr = format!(
            "x='if(gte(t,{in_time}+{delay}),{x_value}{dir_sign}{speed}*(t-({in_time}+{delay})),{x_value})'"
        );
        re.replace(pos, scrolling_expr).to_string()
    } else {
        pos.to_string()
    }
}

/// Returns the width of a character scaled by font size
fn get_char_width(char: char, font_size: f64) -> Option<f64> {
    let width = match char {
        'A' => 0.78,
        'B' => 0.72,
        'C' => 0.70,
        'D' => 0.76,
        'E' => 0.65,
        'F' => 0.65,
        'G' => 0.76,
        'H' => 0.76,
        'I' => 0.30,
        'J' => 0.54,
        'K' => 0.72,
        'L' => 0.60,
        'M' => 0.88,
        'N' => 0.76,
        'O' => 0.82,
        'P' => 0.70,
        'Q' => 0.82,
        'R' => 0.79,
        'S' => 0.70,
        'T' => 0.65,
        'U' => 0.76,
        'V' => 0.72,
        'W' => 0.98,
        'X' => 0.72,
        'Y' => 0.72,
        'Z' => 0.65,
        'a' => 0.60,
        'b' => 0.60,
        'c' => 0.54,
        'd' => 0.60,
        'e' => 0.60,
        'f' => 0.30,
        'g' => 0.60,
        'h' => 0.60,
        'i' => 0.24,
        'j' => 0.24,
        'k' => 0.54,
        'l' => 0.24,
        'm' => 0.88,
        'n' => 0.60,
        'o' => 0.60,
        'p' => 0.60,
        'q' => 0.60,
        'r' => 0.36,
        's' => 0.54,
        't' => 0.30,
        'u' => 0.60,
        'v' => 0.54,
        'w' => 0.76,
        'x' => 0.54,
        'y' => 0.54,
        'z' => 0.54,
        ' ' => 0.50,
        _ => return None,
    };
    Some(width * font_size)
}

// ***before refactoring version***
// use regex::Regex;
// use serde::{Deserialize, Serialize};
// use std::{collections::HashMap, ops::Deref};

// pub trait Effector {
//     fn baker(&self, start: f64, end: f64) -> Option<String>;
// }

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// #[serde(tag = "type", rename_all = "lowercase")]
// pub enum Effect {
//     Fade(FadeArgs),
//     AddText(AddTextArgs),
// }

// impl Effector for Effect {
//     fn baker(&self, start: f64, end: f64) -> Option<String> {
//         match self {
//             Effect::Fade(args) => args.baker(start, end),
//             Effect::AddText(args) => args.baker(start, end),
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// pub struct FadeArgs {
//     fade_variant: String, // "fade_in" or "fade_out"
//     duration: f64,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     alpha: Option<f64>,
// }

// impl Effector for FadeArgs {
//     fn baker(&self, start: f64, end: f64) -> Option<String> {
//         let start = if matches!(self.fade_variant.to_lowercase().deref(), "fade_in" | "in") {
//             Some(start)
//         } else if matches!(self.fade_variant.to_lowercase().deref(), "fade_out" | "out") {
//             Some(end - self.duration)
//         } else {
//             None
//         };

//         let base = match start {
//             Some(start) => {
//                 let fade_type = &self.fade_variant.trim_start_matches("fade_");
//                 format!("fade=t={}:st={}:d={}", fade_type, start, self.duration)
//             }
//             None => return None,
//         };

//         self.alpha
//             .map(|alpha| format!("{}:alpha={}", base, alpha))
//             .or(Some(base))
//     }
// }

// const TEXT_DEFAULT_SIZE: f64 = 20.0;
// const TEXT_DEFAULT_COLOR: &str = "white";
// const TEXT_DEFAULT_TIEM_SHIFT: f64 = 0.0;
// const TEXT_BASE_SCROLLING_SPEED: f64 = 10.0;
// const TEXT_DEFAULT_SCROLL_DELAY: f64 = 5.0;

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// pub struct AddTextArgs {
//     text: String,
//     position: String, // example: x=(W-text_w)/2:y=(H-text_h)/2
//     duration: f64,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     visible_len: Option<String>,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     alpha: Option<f64>,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     font_path: Option<String>,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     #[serde(rename = "t_shift")]
//     time_shift: Option<f64>,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     font_size: Option<f64>,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     font_color: Option<String>,
//     is_shadow: bool,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     shadow_amount: Option<String>,
//     #[serde(rename = "is_fin")]
//     is_fade_in: bool,
//     #[serde(rename = "is_fout")]
//     is_fade_out: bool,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     fade_amount: Option<f64>,
// }

// impl AddTextArgs {
//     fn text_preset(&self, text: String, text_pos: String, start: f64, _end: f64) -> Option<String> {
//         let start = start + self.time_shift.unwrap_or(TEXT_DEFAULT_TIEM_SHIFT);
//         let end = start + self.duration;
//         let position = text_pos;
//         let font_size = &self.font_size.unwrap_or(TEXT_DEFAULT_SIZE);
//         let font_color = &self
//             .font_color
//             .clone()
//             .unwrap_or(TEXT_DEFAULT_COLOR.to_string());

//         let alpha = self.alpha.unwrap_or(1.0);
//         let fade_amount = self.fade_amount.unwrap_or(1.0);
//         let fade_in = if self.is_fade_in {
//             format!(
//             "if(lt(t,{start}),0,if(lt(t,{start}+{fade_amount}),{alpha}*(t-{start})/{fade_amount},{alpha}))"
//             )
//         } else {
//             alpha.to_string()
//         };

//         let fade_out = if self.is_fade_out {
//             format!("if(gt(t,{end}-{fade_amount}),{alpha}*(1-(t-({end}-{fade_amount}))/{fade_amount}),{alpha})")
//         } else {
//             alpha.to_string()
//         };

//         let def_alpha_exp = format!("if(lt(t,{start}),0,if(lt(t,{start}+{fade_amount}),{alpha}*(t-{start})/{fade_amount},{fade_out}))");
//         let alpha_expr = if self.is_fade_in && self.is_fade_out {
//             def_alpha_exp
//         } else if self.is_fade_in {
//             fade_in
//         } else if self.is_fade_out {
//             fade_out
//         } else {
//             alpha.to_string()
//         };

//         let alpha_expr = if self.visible_len.is_some() {
//             let (_, vis_px, vis_marg) =
//                 extract_scroll_components(&self.visible_len.clone().unwrap());
//             let (def_pos_x, _) = extract_position_components(&self.position);
//             format!(
//                 "if(lt(x,{def_pos_x}-tw/2),if(gt(x,{def_pos_x}-{vis_marg}),{alpha}-(({alpha}/{vis_marg})*(({def_pos_x})-x)),0),if(gt(x,{def_pos_x}+{vis_px}),if(gt(x,{def_pos_x}+{vis_px}+{vis_marg}),0,({alpha}/{vis_marg})*({def_pos_x}+{vis_px}+{vis_marg}-x)),{alpha_expr}))")
//         } else {
//             alpha_expr
//             // without scrolling limitation
//         };

//         let shadow = if self.is_shadow {
//             let raw_shdw_val = self.shadow_amount.clone().unwrap_or("x=2, y=2".to_string());
//             let re = Regex::new(r"x=([^:]+):y=([^:]+)").unwrap();
//             let captures = re.captures(&raw_shdw_val)?;
//             let shdw_x = captures.get(1)?.as_str();
//             let shdw_y = captures.get(2)?.as_str();
//             format!(":shadowx={shdw_x}:shadowy={shdw_y}")
//         } else {
//             "".to_string()
//         };

//         let font_path = self
//             .font_path
//             .clone()
//             .map(|file_path| format!("fontfile={}:", file_path))
//             .unwrap_or_default();

//         let filter = format!(
//             "drawtext=text='{text}':{position}:fontsize={font_size}:{font_path}fontcolor={font_color}{shadow}:alpha='{alpha_expr}':enable='between(t,{start},{end})'"
//         );

//         Some(filter)
//     }
// }

// impl Effector for AddTextArgs {
//     fn baker(&self, start: f64, end: f64) -> Option<String> {
//         if self.visible_len.is_some() {
//             let uppercase_text = self.text.to_uppercase();
//             let mut txt_filters = Vec::new();
//             let mut current_pos_x = 0.0;

//             for char in uppercase_text.chars() {
//                 // todo: handle the "'" existed in the text
//                 if char == '\'' {
//                     continue;
//                 }
//                 let char_pos = {
//                     let previous_char_width =
//                         get_char_width(char, self.font_size.unwrap_or(TEXT_DEFAULT_SIZE))
//                             .unwrap_or(TEXT_DEFAULT_SIZE * 0.6) as f64;
//                     let (pos_x, pos_y) = extract_position_components(&self.position);
//                     let new_x = format!("x={}+{}", pos_x, current_pos_x);
//                     let new_y = format!("y={}", pos_y);
//                     let new_pos = format!("{}:{}", new_x, new_y);

//                     current_pos_x += previous_char_width * 0.9;
//                     let (scroll_dir, _, _) =
//                         extract_scroll_components(&self.visible_len.clone().unwrap());
//                     scrolling(
//                         start,
//                         &new_pos,
//                         TEXT_BASE_SCROLLING_SPEED + 18.0,
//                         TEXT_DEFAULT_SCROLL_DELAY,
//                         scroll_dir,
//                     )
//                 };

//                 let char_filter =
//                     self.text_preset(char.escape_debug().to_string(), char_pos, start, end);
//                 txt_filters.push(char_filter);
//             }
//             let txt_filter = txt_filters
//                 .into_iter()
//                 .flatten()
//                 .collect::<Vec<_>>()
//                 .join(",");
//             Some(txt_filter)
//         } else {
//             self.text_preset(self.text.to_uppercase(), self.position.clone(), start, end)
//         }
//     }
// }

// fn extract_position_components(position: &str) -> (String, String) {
//     let re = Regex::new(r"x=([^:]+):y=([^:]+)").expect("Invalid regex pattern");
//     if let Some(captures) = re.captures(position) {
//         let x_value = captures[1].to_string();
//         let y_value = captures[2].to_string();
//         (x_value, y_value)
//     } else {
//         ("0".to_string(), "0".to_string())
//     }
// }

// fn extract_scroll_components(scroll_elements: &str) -> (char, String, String) {
//     let re = Regex::new(r"dir=([LR]):vis_px=([^:]+):margin_px=([^:]+)") // dir=LR is to left and to right
//         .expect("Invalid regex pattern");
//     if let Some(captures) = re.captures(scroll_elements) {
//         let direction = captures[2].chars().next().unwrap();
//         let vis_val = captures[2].to_string();
//         let marg_val = captures[3].to_string();
//         (direction, vis_val, marg_val)
//     } else {
//         ('n', "0".to_string(), "0".to_string())
//     }
// }

// fn scrolling(in_time: f64, pos: &str, speed: f64, delay: f64, dir: char) -> String {
//     let re = Regex::new(r"x=([^:]+)").expect("Invalid regex pattern");
//     if let Some(captures) = re.captures(pos) {
//         let x_value = &captures[1];
//         let dir_sign = match dir {
//             'L' => "-",
//             'R' => "+",
//             _ => "-", // default to left
//         };

//         let scrolling_expr = format!(
//             "x='if(gte(t,{in_time}+{delay}),{x_value}{dir_sign}{speed}*(t-({in_time}+{delay})),{x_value})'",
//         );
//         re.replace(pos, scrolling_expr).to_string()
//     } else {
//         pos.to_string()
//     }
// }

// fn get_char_width(char: char, font_size: f64) -> Option<f64> {
//     let mut width_table: HashMap<char, f64> = HashMap::new();

//     // Uppercase letters (A-Z)
//     width_table.insert('A', 0.78);
//     width_table.insert('B', 0.72);
//     width_table.insert('C', 0.70);
//     width_table.insert('D', 0.76);
//     width_table.insert('E', 0.65);
//     width_table.insert('F', 0.65);
//     width_table.insert('G', 0.76);
//     width_table.insert('H', 0.76);
//     width_table.insert('I', 0.30);
//     width_table.insert('J', 0.54);
//     width_table.insert('K', 0.72);
//     width_table.insert('L', 0.60);
//     width_table.insert('M', 0.88);
//     width_table.insert('N', 0.76);
//     width_table.insert('O', 0.82);
//     width_table.insert('P', 0.70);
//     width_table.insert('Q', 0.82);
//     width_table.insert('R', 0.79);
//     width_table.insert('S', 0.70);
//     width_table.insert('T', 0.65);
//     width_table.insert('U', 0.76);
//     width_table.insert('V', 0.72);
//     width_table.insert('W', 0.98);
//     width_table.insert('X', 0.72);
//     width_table.insert('Y', 0.72);
//     width_table.insert('Z', 0.65);

//     // Lowercase letters (a-z)
//     width_table.insert('a', 0.60);
//     width_table.insert('b', 0.60);
//     width_table.insert('c', 0.54);
//     width_table.insert('d', 0.60);
//     width_table.insert('e', 0.60);
//     width_table.insert('f', 0.30);
//     width_table.insert('g', 0.60);
//     width_table.insert('h', 0.60);
//     width_table.insert('i', 0.24);
//     width_table.insert('j', 0.24);
//     width_table.insert('k', 0.54);
//     width_table.insert('l', 0.24);
//     width_table.insert('m', 0.88);
//     width_table.insert('n', 0.60);
//     width_table.insert('o', 0.60);
//     width_table.insert('p', 0.60);
//     width_table.insert('q', 0.60);
//     width_table.insert('r', 0.36);
//     width_table.insert('s', 0.54);
//     width_table.insert('t', 0.30);
//     width_table.insert('u', 0.60);
//     width_table.insert('v', 0.54);
//     width_table.insert('w', 0.76);
//     width_table.insert('x', 0.54);
//     width_table.insert('y', 0.54);
//     width_table.insert('z', 0.54);

//     // Space
//     width_table.insert(' ', 0.50);

//     width_table.get(&char).map(|&width| width * font_size)
// }
