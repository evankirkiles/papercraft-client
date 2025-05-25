#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum MSAALevel {
    None,
    #[default]
    X4,
}

impl From<&MSAALevel> for u32 {
    fn from(val: &MSAALevel) -> Self {
        match val {
            MSAALevel::None => 1,
            MSAALevel::X4 => 4,
        }
    }
}

/// User-configurable settings for the draw state
#[derive(Debug, Default)]
pub struct Settings {
    pub msaa_level: MSAALevel,
}
