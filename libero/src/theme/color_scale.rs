#[derive(Clone, PartialEq, Eq)]
pub struct ColorScale {
    colors: [String; 10],
}

impl ColorScale {
    pub fn new(colors: [String; 10]) -> Self {
        Self { colors }
    }

    pub fn from_anchor(hex: &str, index: usize) -> Self {
        assert!(
            index < 10,
            "color scale anchor index must be between 0 and 9"
        );

        let (red, green, blue) =
            parse_hex_color(hex).unwrap_or_else(|| panic!("invalid hex color: {hex}"));

        let mut colors: [String; 10] = std::array::from_fn(|_| String::new());

        for (shade_index, color) in colors.iter_mut().enumerate() {
            let distance = shade_index as f32 - index as f32;
            let normalized = distance / 9.0;

            let (target_red, target_green, target_blue, mix) = if normalized < 0.0 {
                (255.0, 255.0, 255.0, -normalized)
            } else {
                (0.0, 0.0, 0.0, normalized)
            };

            let mixed_red = mix_channel(red as f32, target_red, mix);
            let mixed_green = mix_channel(green as f32, target_green, mix);
            let mixed_blue = mix_channel(blue as f32, target_blue, mix);

            *color = format!("#{:02X}{:02X}{:02X}", mixed_red, mixed_green, mixed_blue);
        }

        Self { colors }
    }

    pub fn get(&self, index: usize) -> &str {
        &self.colors[index]
    }

    pub fn colors(&self) -> &[String; 10] {
        &self.colors
    }
}

fn mix_channel(base: f32, target: f32, ratio: f32) -> u8 {
    let ratio = ratio.clamp(0.0, 1.0);
    let value = base + (target - base) * ratio;
    value.round().clamp(0.0, 255.0) as u8
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);

    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((red, green, blue))
}
