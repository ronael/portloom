//! Presentation-only frame selection; artwork and timing stay independent of the terminal.
mod bat;

pub type Pixels = [[u8; 16]; 16];
pub struct SpriteFrame {
    pub duration: u64,
    pub pixels: Pixels,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sequence {
    Arrival,
    Rest,
    Listening,
    Alert,
    Departure,
}
impl Sequence {
    pub fn frames(self) -> &'static [SpriteFrame] {
        match self {
            Self::Arrival => bat::ARRIVAL,
            Self::Rest => bat::REST,
            Self::Listening => bat::LISTENING,
            Self::Alert => bat::ALERT,
            Self::Departure => bat::DEPARTURE,
        }
    }
    pub fn duration(self) -> u64 {
        self.frames().iter().map(|frame| frame.duration).sum()
    }
    pub fn pixels(self, elapsed_ms: u64) -> &'static Pixels {
        let duration = self.duration();
        let mut remaining = if matches!(self, Self::Arrival | Self::Departure) {
            elapsed_ms.min(duration - 1)
        } else {
            elapsed_ms % duration
        };
        for frame in self.frames() {
            if remaining < frame.duration {
                return &frame.pixels;
            }
            remaining -= frame.duration;
        }
        &self.frames()[0].pixels
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarFrame {
    pub mood: Mood,
    pub pixels: &'static Pixels,
}
impl Default for AvatarFrame {
    fn default() -> Self {
        Self::at(Mood::Idle, false, 0)
    }
}
impl AvatarFrame {
    pub fn at(mood: Mood, motion: bool, elapsed_ms: u64) -> Self {
        let arrival = Sequence::Arrival.duration();
        let pixels = if mood == Mood::Paused {
            // The approved resting sequence includes a closed-eye pose.
            &bat::REST[8].pixels
        } else if !motion {
            &bat::REST[0].pixels
        } else if matches!(mood, Mood::Idle | Mood::Working) && elapsed_ms < arrival {
            Sequence::Arrival.pixels(elapsed_ms)
        } else {
            match mood {
                Mood::Working | Mood::Focused => Sequence::Listening.pixels(elapsed_ms),
                Mood::Warning | Mood::Danger => Sequence::Alert.pixels(elapsed_ms),
                _ => Sequence::Rest.pixels(elapsed_ms.saturating_sub(arrival)),
            }
        };
        Self { mood, pixels }
    }
    pub fn departing(elapsed_ms: u64) -> Self {
        Self {
            mood: Mood::Idle,
            pixels: Sequence::Departure.pixels(elapsed_ms),
        }
    }
}
