use portloom::{
    domain::policy::Actor,
    presentation::{
        animation::{AvatarFrame, Mood, Pose},
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
    assert_eq!(AvatarFrame::at(Mood::Idle, true, 3250).pose, Pose::Blink);
    assert_ne!(
        AvatarFrame::at(Mood::Working, true, 0),
        AvatarFrame::at(Mood::Working, true, 180)
    );
    assert_eq!(
        AvatarFrame::at(Mood::Paused, true, 0),
        AvatarFrame::at(Mood::Paused, true, 9999)
    );
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
