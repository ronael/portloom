use portloom::{
    domain::policy::Actor,
    presentation::{
        animation::{AvatarFrame, Mood, Sequence},
        state::{Overlay, State},
    },
};

#[test]
fn motion_has_still_intervals_and_reduced_motion_is_time_independent() {
    for mood in [
        Mood::Idle,
        Mood::Working,
        Mood::Paused,
        Mood::Focused,
        Mood::Warning,
        Mood::Danger,
    ] {
        let still = AvatarFrame::at(mood, false, 0);
        for time in [1, 180, 3300, 7000, u64::MAX] {
            assert_eq!(AvatarFrame::at(mood, false, time), still);
        }
    }
    assert_eq!(
        AvatarFrame::at(Mood::Idle, true, 1500),
        AvatarFrame::at(Mood::Idle, true, 3000)
    );
    assert_ne!(
        AvatarFrame::at(Mood::Working, true, 2000),
        AvatarFrame::at(Mood::Working, true, 2200)
    );
    assert_eq!(
        AvatarFrame::at(Mood::Paused, true, 0),
        AvatarFrame::at(Mood::Paused, true, 9999)
    );
}

#[test]
fn approved_sequences_keep_their_durations_and_one_shot_endings() {
    for (sequence, count, duration) in [
        (Sequence::Arrival, 15, 1280),
        (Sequence::Rest, 10, 2000),
        (Sequence::Listening, 9, 1500),
        (Sequence::Alert, 10, 840),
        (Sequence::Departure, 15, 1200),
    ] {
        assert_eq!(sequence.frames().len(), count);
        assert_eq!(sequence.duration(), duration);
        assert!(sequence.frames().iter().all(|frame| {
            frame.duration > 0
                && frame
                    .pixels
                    .iter()
                    .flatten()
                    .all(|pixel| b".PLSD".contains(pixel))
        }));
        assert_eq!(sequence.pixels(u64::MAX).len(), 16);
    }
    assert_eq!(Sequence::Arrival.pixels(u64::MAX), Sequence::Rest.pixels(0));
    assert!(
        Sequence::Departure
            .pixels(u64::MAX)
            .iter()
            .flatten()
            .all(|pixel| *pixel == b'.')
    );
    assert_eq!(Sequence::Rest.pixels(0), Sequence::Rest.pixels(2000));
}

#[test]
fn animation_only_requests_redraw_for_changes_and_freezes_behind_dialogs() {
    let mut state = State::new(
        Actor {
            uid: 501,
            protected_pids: vec![],
        },
        true,
    );
    assert!(state.animate(0));
    assert!(!state.animate(50));
    assert!(state.animate(180));
    state.overlay = Overlay::Help;
    state.animate(200);
    assert!(!state.animate(800));
    assert!(!state.animate(3300));
    state.overlay = Overlay::None;
    state.motion_enabled = false;
    state.animate(4000);
    assert!(!state.animate(5000));
    state.stale = true;
    assert!(state.animate(5001));
    assert_eq!(state.avatar_frame.mood, Mood::Danger);
}
