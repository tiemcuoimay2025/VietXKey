// Regressions for literal escapes, capitalized Vietnamese and native/TSF delivery.
use vietflow_core::{CharsetId, CustomKeymap, EngineOptions, InputMethod, VietFlowEngine};

fn options(method: InputMethod) -> EngineOptions {
    EngineOptions {
        method,
        custom_keymap: CustomKeymap::telex(),
        ..Default::default()
    }
}

fn render(engine: &mut VietFlowEngine, input: &str) -> String {
    let mut screen = String::new();
    for c in input.chars() {
        let (backspaces, output) = engine.process_key(c);
        for _ in 0..backspaces {
            assert!(
                screen.pop().is_some(),
                "over-deletion in {input:?} at {c:?}"
            );
        }
        screen.push_str(output.as_str());
        assert_ne!(engine.take_auto_switch_request(), Some(false));
    }
    screen
}

#[test]
fn windows_with_one_w_still_works_in_every_method() {
    for method in [
        InputMethod::Telex,
        InputMethod::SimpleTelex,
        InputMethod::TelexSimple2,
        InputMethod::TelexVni,
        InputMethod::Vni,
        InputMethod::Viqr,
        InputMethod::UserCustom,
    ] {
        let mut engine = VietFlowEngine::new(options(method), CharsetId::Unicode);
        assert_eq!(
            render(&mut engine, "Windows windows WINDOWS "),
            "Windows windows WINDOWS ",
            "{method:?}"
        );
    }
}

#[test]
fn escaped_w_stays_single_at_every_delimiter_and_option_combination() {
    for method in [
        InputMethod::Telex,
        InputMethod::TelexVni,
        InputMethod::UserCustom,
    ] {
        for auto in [false, true] {
            for spell_check in [false, true] {
                for free_marking in [false, true] {
                    for (input, expected) in [
                        ("WWindows", "Windows"),
                        ("Wwindows", "Windows"),
                        ("wwindows", "windows"),
                        ("wword", "word"),
                    ] {
                        for delimiter in [' ', ',', '.', '!', '?', '\n', '\t'] {
                            let mut engine = VietFlowEngine::new(
                                EngineOptions {
                                    auto_switch_by_typing: auto,
                                    spell_check,
                                    free_marking,
                                    ..options(method)
                                },
                                CharsetId::Unicode,
                            );
                            // Verify before and after committing, including the exact W count.
                            let mut screen = render(&mut engine, input);
                            assert_eq!(screen, expected, "{method:?}: {input}");
                            let (backspaces, output) = engine.process_key(delimiter);
                            for _ in 0..backspaces {
                                assert!(screen.pop().is_some());
                            }
                            screen.push_str(output.as_str());
                            assert_eq!(
                                screen,
                                format!("{expected}{delimiter}"),
                                "{method:?}: {input}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn explicit_undo_survives_word_break_and_following_english() {
    for (input, expected) in [
        ("WWindows was tieengs Vieetj ", "Windows was tiếng Việt "),
        ("WWindows, DDaf Nawngx! ", "Windows, Đà Nẵng! "),
        ("WWindows\x08s DDaf Nawngx ", "Windows Đà Nẵng "),
        ("ww\x08DDaf ", "Đà "),
        ("aaapple ", "aapple "),
        ("eeemail ", "eemail "),
        ("ooops ", "oops "),
        ("ddddata ", "dddata "),
        ("uoww ", "uow "),
        ("baww ", "baw "),
        ("bangww ", "bangw "),
    ] {
        let mut engine = VietFlowEngine::default();
        assert_eq!(render(&mut engine, input), expected, "{input:?}");
    }
}

#[test]
fn da_nang_keeps_accents_with_both_capital_d_keys() {
    for method in [
        InputMethod::Telex,
        InputMethod::SimpleTelex,
        InputMethod::TelexSimple2,
        InputMethod::TelexVni,
        InputMethod::UserCustom,
    ] {
        for auto in [false, true] {
            for (input, expected) in [
                ("DDaf Nawngx ", "Đà Nẵng "),
                ("Ddaf Nawngx ", "Đà Nẵng "),
                ("ddaf nawngx ", "đà nẵng "),
                ("DDAF NAWNGX ", "ĐÀ NẴNG "),
                ("DDawng DDuwcs DDoo DDaf. ", "Đăng Đức Đô Đà. "),
                ("Windows DDaf Nawngx Windows ", "Windows Đà Nẵng Windows "),
            ] {
                let mut engine = VietFlowEngine::new(
                    EngineOptions {
                        auto_switch_by_typing: auto,
                        ..options(method)
                    },
                    CharsetId::Unicode,
                );
                assert_eq!(render(&mut engine, input), expected, "{method:?}: {input}");
            }
        }
    }
}

#[test]
fn vni_escapes_and_capital_d_are_preserved() {
    for (input, expected) in [
        ("D9a2 Na8ng4 ", "Đà Nẵng "),
        ("a66 ", "a6 "),
        ("a88 ", "a8 "),
        ("d99 ", "d9 "),
    ] {
        let mut engine = VietFlowEngine::new(options(InputMethod::Vni), CharsetId::Unicode);
        assert_eq!(render(&mut engine, input), expected, "{input}");
    }
}

#[test]
fn tsf_finalization_preserves_escaped_and_vietnamese_preedit() {
    for (input, expected) in [
        ("WWindows", "Windows"),
        ("Wword", "Word"),
        ("DDaf", "Đà"),
        ("Nawngx", "Nẵng"),
        ("ooo", "oo"),
    ] {
        for delimiter in [' ', '.', '\n', '\t'] {
            let mut engine = VietFlowEngine::default();
            for c in input.chars() {
                let _ = engine.process_key(c);
            }
            assert_eq!(engine.current_preedit_text(), expected, "{input}");
            assert_eq!(
                engine.finish_composition_on_break(delimiter),
                expected,
                "{input}"
            );
            assert!(engine.is_empty());
            assert_eq!(render(&mut engine, "tieengs Vieetj "), "tiếng Việt ");
        }
    }
}

#[test]
fn literal_escape_preserves_preedit_in_legacy_and_composite_charsets() {
    for charset in [
        CharsetId::Unicode,
        CharsetId::UnicodeComposite,
        CharsetId::Tcvn3,
        CharsetId::VniWindows,
        CharsetId::Viqr,
    ] {
        for input in ["WWindows", "uoww", "ddd", "ooo"] {
            let mut engine = VietFlowEngine::new(EngineOptions::default(), charset);
            let before = render(&mut engine, input);
            let (backspaces, output) = engine.process_key(' ');
            assert_eq!(backspaces, 0, "{charset:?}: {input}");
            assert_eq!(output.as_str(), " ");
            let mut tsf = VietFlowEngine::new(EngineOptions::default(), charset);
            for c in input.chars() {
                let _ = tsf.process_key(c);
            }
            assert_eq!(
                tsf.finish_composition_on_break(' '),
                before,
                "{charset:?}: {input}"
            );
        }
    }
}

#[test]
fn uppercase_vietnamese_scoring_does_not_confuse_action_keys_with_capitals() {
    use vietflow_core::autoswitch::{
        should_restore_to_english, should_switch_to_vietnamese, to_telex_normalized_key,
    };
    for (input, key) in [
        ("Đà", "ddaf"),
        ("ĐÀ", "ddaf"),
        ("NẴNG", "nawngx"),
        ("ĐỨC", "dduwcs"),
        ("TIẾNG", "tieengs"),
    ] {
        assert_eq!(
            to_telex_normalized_key(input).as_deref(),
            Some(key),
            "{input}"
        );
    }
    assert!(!should_restore_to_english("DDaf", "Đà", None, true));
    assert!(should_switch_to_vietnamese("DDaf", "Đà", None));
    assert!(should_restore_to_english(
        "getUserId",
        "getUserId",
        None,
        false
    ));
    for input in ["getUserId", "HTML", "NASA", "USA", "XMLHttpRequest"] {
        let mut engine = VietFlowEngine::default();
        assert_eq!(
            render(&mut engine, &format!("{input} ")),
            format!("{input} ")
        );
    }
}
