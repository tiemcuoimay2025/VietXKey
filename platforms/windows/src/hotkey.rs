// Modifier-only shortcuts must finish before they can be distinguished from
// application shortcuts such as Ctrl+Shift+X. No OS calls or allocation here.
pub const CTRL: u8 = 1;
pub const SHIFT: u8 = 2;
pub const ALT: u8 = 4;
pub const WIN: u8 = 8;

pub fn modifier_bit(vk: u16) -> u8 {
    match vk {
        0x11 | 0xA2 | 0xA3 => CTRL,
        0x10 | 0xA0 | 0xA1 => SHIFT,
        0x12 | 0xA4 | 0xA5 => ALT,
        0x5B | 0x5C => WIN,
        _ => 0,
    }
}

pub fn modifier_toggle_mask(mode: &str) -> Option<u8> {
    let mode = mode.strip_prefix("Custom:").unwrap_or(mode).trim();
    let compact_eq = |expected: &str| {
        mode.chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_lowercase())
            .eq(expected.chars().map(|c| c.to_ascii_lowercase()))
    };
    if compact_eq("CtrlShift") {
        return Some(CTRL | SHIFT);
    }
    if compact_eq("AltShift") {
        return Some(ALT | SHIFT);
    }
    let mut mask = 0;
    for part in mode.split('+').map(str::trim) {
        mask |= if part.eq_ignore_ascii_case("ctrl") || part.eq_ignore_ascii_case("control") {
            CTRL
        } else if part.eq_ignore_ascii_case("shift") {
            SHIFT
        } else if part.eq_ignore_ascii_case("alt") {
            ALT
        } else if part.eq_ignore_ascii_case("win") || part.eq_ignore_ascii_case("windows") {
            WIN
        } else {
            return None;
        };
    }
    (mask != 0).then_some(mask)
}

#[derive(Default)]
pub struct ModifierChordTracker {
    required: u8,
    armed: bool,
    cancelled: bool,
    last_sequence: u64,
}

impl ModifierChordTracker {
    fn cancel(&mut self) {
        self.armed = false;
        self.cancelled = true;
    }

    pub fn on_focus_change(&mut self) {
        // Discard an unfinished chord, but do not block the first clean
        // shortcut when focus changed while no modifiers were held.
        self.armed = false;
    }

    /// `modifiers` is the physical state AFTER this event. `other_key_down`
    /// includes keys already held before the chord, not just its final trigger.
    pub fn update(
        &mut self,
        required: u8,
        vk: u16,
        down: bool,
        modifiers: u8,
        other_key_down: bool,
        sequence: u64,
    ) -> bool {
        if self.last_sequence != 0
            && (sequence != self.last_sequence.wrapping_add(1) || self.required != required)
        {
            // Missed input (e.g. busy hook mutex) or a changed setting must
            // never turn an incomplete gesture into an accidental toggle.
            self.cancel();
        }
        self.last_sequence = sequence;
        self.required = required;
        if required == 0 {
            self.armed = false;
            self.cancelled = modifiers != 0;
            return false;
        }
        if other_key_down || modifiers & !required != 0 {
            self.cancel();
        }
        let fired = !down
            && modifier_bit(vk) & required != 0
            && modifiers & required != required
            && self.armed
            && !self.cancelled;
        if fired {
            self.armed = false;
        }
        if modifiers == 0 {
            self.armed = false;
            self.cancelled = false;
        } else if down && modifier_bit(vk) != 0 && modifiers == required && !self.cancelled {
            self.armed = true;
        }
        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const C: u16 = 0xA2;
    const S: u16 = 0xA0;
    const CS: u8 = CTRL | SHIFT;

    fn events(required: u8, input: &[(u16, bool, u8, bool)]) -> Vec<usize> {
        let mut tracker = ModifierChordTracker::default();
        input
            .iter()
            .enumerate()
            .filter_map(|(i, &(vk, down, mods, other))| {
                tracker
                    .update(required, vk, down, mods, other, i as u64 + 1)
                    .then_some(i)
            })
            .collect()
    }

    #[test]
    fn ctrl_shift_fires_on_release_once_in_either_order() {
        for (first, second, first_mask, remaining) in [(C, S, CTRL, CTRL), (S, C, SHIFT, SHIFT)] {
            assert_eq!(
                events(
                    CS,
                    &[
                        (first, true, first_mask, false),
                        (second, true, CS, false),
                        (second, true, CS, false),
                        (second, false, remaining, false),
                        (first, false, 0, false)
                    ]
                ),
                vec![3]
            );
            assert_eq!(
                events(
                    CS,
                    &[
                        (first, true, first_mask, false),
                        (second, true, CS, false),
                        (first, false, CS ^ first_mask, false),
                        (second, false, 0, false)
                    ]
                ),
                vec![2]
            );
        }
    }

    #[test]
    fn third_key_cancels_until_modifiers_are_released() {
        // Letters, settings hotkey F5, function keys, arrows and editing keys.
        for key in (0x41..=0x5A)
            .chain(0x70..=0x7B)
            .chain([0x08, 0x09, 0x20, 0x25, 0x27, 0x2E])
        {
            assert!(
                events(
                    CS,
                    &[
                        (C, true, CTRL, false),
                        (S, true, CS, false),
                        (key, true, CS, true),
                        (key, false, CS, false),
                        (S, false, CTRL, false),
                        (C, false, 0, false)
                    ]
                )
                .is_empty(),
                "VK {key:X}"
            );
            assert!(
                events(
                    CS,
                    &[
                        (C, true, CTRL, false),
                        (S, true, CS, false),
                        (key, true, CS, true),
                        (C, false, SHIFT, true),
                        (S, false, 0, true),
                        (key, false, 0, false)
                    ]
                )
                .is_empty(),
                "release VK {key:X}"
            );
        }
    }

    #[test]
    fn already_held_key_and_extra_modifiers_do_not_toggle() {
        assert!(events(
            CS,
            &[
                (0x58, true, 0, true),
                (C, true, CTRL, true),
                (S, true, CS, true),
                (0x58, false, CS, false),
                (S, false, CTRL, false),
                (C, false, 0, false)
            ]
        )
        .is_empty());
        for (key, bit) in [(0xA4, ALT), (0x5B, WIN)] {
            assert!(events(
                CS,
                &[
                    (C, true, CTRL, false),
                    (S, true, CS, false),
                    (key, true, CS | bit, false),
                    (key, false, CS, false),
                    (S, false, CTRL, false),
                    (C, false, 0, false)
                ]
            )
            .is_empty());
        }
    }

    #[test]
    fn clean_chord_recovers_after_cancelled_shortcut() {
        assert_eq!(
            events(
                CS,
                &[
                    (C, true, CTRL, false),
                    (S, true, CS, false),
                    (0x58, true, CS, true),
                    (0x58, false, CS, false),
                    (S, false, CTRL, false),
                    (C, false, 0, false),
                    (C, true, CTRL, false),
                    (S, true, CS, false),
                    (C, false, SHIFT, false),
                    (S, false, 0, false)
                ]
            ),
            vec![8]
        );
    }

    #[test]
    fn skipped_events_focus_and_configuration_changes_cancel() {
        for gap in [true, false] {
            let mut tracker = ModifierChordTracker::default();
            assert!(!tracker.update(CS, C, true, CTRL, false, 1));
            assert!(!tracker.update(CS, S, true, CS, false, 2));
            if !gap {
                tracker.cancel();
            }
            assert!(!tracker.update(CS, S, false, CTRL, false, if gap { 4 } else { 3 }));
        }
        let mut tracker = ModifierChordTracker::default();
        tracker.update(CS, C, true, CTRL, false, 1);
        tracker.update(CS, S, true, CS, false, 2);
        assert!(!tracker.update(CTRL, S, false, CTRL, false, 3));
        assert!(!tracker.update(CTRL, C, false, 0, false, 4));
    }

    #[test]
    fn alt_shift_and_generic_right_modifiers_use_same_guard() {
        assert_eq!(
            events(
                ALT | SHIFT,
                &[
                    (0xA5, true, ALT, false),
                    (0xA1, true, ALT | SHIFT, false),
                    (0xA5, false, SHIFT, false),
                    (0xA1, false, 0, false)
                ]
            ),
            vec![2]
        );
        assert_eq!(
            events(
                CS,
                &[
                    (0x11, true, CTRL, false),
                    (0xA1, true, CS, false),
                    (0x11, false, SHIFT, false),
                    (0xA1, false, 0, false)
                ]
            ),
            vec![2]
        );
    }

    #[test]
    fn focus_change_keeps_the_first_clean_shortcut_usable() {
        let mut tracker = ModifierChordTracker::default();
        tracker.on_focus_change();
        assert!(!tracker.update(CS, C, true, CTRL, false, 1));
        assert!(!tracker.update(CS, S, true, CS, false, 2));
        tracker.on_focus_change();
        assert!(!tracker.update(CS, S, false, CTRL, false, 3));
        assert!(!tracker.update(CS, C, false, 0, false, 4));
        assert!(!tracker.update(CS, C, true, CTRL, false, 5));
        assert!(!tracker.update(CS, S, true, CS, false, 6));
        assert!(tracker.update(CS, S, false, CTRL, false, 7));
    }

    #[test]
    fn parses_modifier_presets_and_custom_aliases_without_matching_keyed_shortcuts() {
        for mode in [
            "CtrlShift",
            "ctrlshift",
            "Ctrl + Shift",
            "Custom:Control + Shift",
        ] {
            assert_eq!(modifier_toggle_mask(mode), Some(CS), "{mode}");
        }
        assert_eq!(modifier_toggle_mask("AltShift"), Some(ALT | SHIFT));
        assert_eq!(
            modifier_toggle_mask("Ctrl + Alt + Shift"),
            Some(CTRL | ALT | SHIFT)
        );
        for mode in [
            "",
            "CtrlSpace",
            "Alt + Z",
            "Ctrl + Shift + X",
            "Ctrl + Shift + F5",
        ] {
            assert_eq!(modifier_toggle_mask(mode), None, "{mode}");
        }
    }
}
