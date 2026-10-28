//! Pure presentation timing: only a changed pose asks the terminal to redraw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mood {
    #[default]
    Idle,
    Working,
    Paused,
    Focused,
    Warning,
    Danger,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pose {
    #[default]
    Still,
    Blink,
    Wink,
    LookLeft,
    LookRight,
    WaveHigh,
    WaveLow,
    StepLeft,
    StepRight,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AvatarFrame {
    pub mood: Mood,
    pub pose: Pose,
    pub pulse: bool,
}
impl AvatarFrame {
    pub fn at(mood: Mood, motion: bool, elapsed_ms: u64) -> Self {
        let pose = match mood {
            Mood::Paused => Pose::Blink,
            Mood::Focused => Pose::LookRight,
            Mood::Working if motion => match (elapsed_ms / 180) % 4 {
                0 => Pose::StepLeft,
                1 => Pose::LookLeft,
                2 => Pose::StepRight,
                _ => Pose::LookRight,
            },
            Mood::Working => Pose::LookLeft,
            Mood::Idle if motion => match elapsed_ms % 10_000 {
                0..160 | 320..480 | 640..800 => Pose::WaveLow,
                160..320 | 480..640 => Pose::WaveHigh,
                800..1100 => Pose::Wink,
                3200..3320 | 3440..3560 => Pose::Blink,
                6000..6400 => Pose::LookLeft,
                6800..7200 => Pose::LookRight,
                _ => Pose::Still,
            },
            _ => Pose::Still,
        };
        Self {
            mood,
            pose,
            pulse: motion && mood == Mood::Working && (elapsed_ms / 180).is_multiple_of(2),
        }
    }
}
