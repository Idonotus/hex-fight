use std::{cmp::PartialOrd, fmt::Display};

#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub(crate) struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

fn get_max_val(vals: &[f32; 3]) -> f32 {
    vals[0].max(vals[1].max(vals[2]))
}

fn get_min_val(vals: &[f32; 3]) -> f32 {
    vals[0].min(vals[1].min(vals[2]))
}

impl Color {
    pub fn get_float_vals(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    pub fn get_value(&self) -> f32 {
        get_max_val(&self.get_float_vals())
    }

    pub fn get_saturation(&self) -> f32 {
        let vals = self.get_float_vals();
        let max_val = get_max_val(&vals);
        let min_val = get_min_val(&vals);
        if max_val == 0f32 {
            return 0f32;
        }
        (max_val - min_val) / max_val
    }

    pub fn get_hue(&self) -> f32 {
        let vals = self.get_float_vals();
        let max_val = get_max_val(&vals);
        let min_val = get_min_val(&vals);
        let delta = max_val - min_val;
        if delta == 0f32 {
            return 0f32;
        }
        if max_val == vals[0] {
            60f32 * ((vals[1] - vals[2]) / delta % 6f32)
        } else if max_val == vals[1] {
            60f32 * ((vals[2] - vals[0]) / delta + 2f32)
        } else {
            60f32 * ((vals[0] - vals[1]) / delta + 4f32)
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

pub type ColorComparison<'a> = Box<dyn Fn(Color, Color) -> bool + 'a>;
pub type ColorDifferenceFn<T> = fn(Color, Color) -> T;

pub fn exact_color_eq() -> ColorComparison<'static> {
    Box::new(|a: Color, b: Color| -> bool { a == b })
}

pub fn build_compare_tolerance<'a, T: PartialOrd + 'a>(
    distance_func: ColorDifferenceFn<T>,
    tolorance: T,
) -> ColorComparison<'a> {
    return Box::new(move |a: Color, b: Color| distance_func(a, b) <= tolorance);
}

pub fn taxicab(a: Color, b: Color) -> u16 {
    let r_delta: u16 = a.r.abs_diff(b.r).into();
    let g_delta: u16 = a.b.abs_diff(b.b).into();
    let b_delta: u16 = a.g.abs_diff(b.g).into();

    return r_delta + g_delta + b_delta;
}
