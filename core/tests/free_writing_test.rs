use vietflow_core::{CharsetId, CustomKeymap, EngineOptions, InputMethod, VietFlowEngine};
fn feed(engine: &mut VietFlowEngine, input: &str) -> String {
    let mut text = String::new();
    for c in input.chars() {
        let (remove, output) = engine.process_key(c);
        for _ in 0..remove {
            assert!(text.pop().is_some(), "over-delete: {input}");
        }
        text.push_str(output.as_str());
        assert_ne!(engine.take_auto_switch_request(), Some(false));
    }
    text
}
fn options(enabled: bool) -> EngineOptions {
    EngineOptions {
        preserve_informal_marks: enabled,
        ..Default::default()
    }
}
#[test]
fn opt_in_preserves_visible_spelling_without_changing_existing_defaults() {
    assert!(!EngineOptions::default().preserve_informal_marks);
    for (input, regular, free) in [
        ("of ", "of ", "ò "),
        ("test ", "test ", "tét "),
        ("cafp ", "cafp ", "càp "),
        ("capf ", "capf ", "càp "),
        ("aafbc ", "aafbc ", "ầbc "),
        ("aabcf ", "aabcf ", "ầbc "),
        ("ofh zoo ", "òh zô ", "òh zô "),
    ] {
        assert_eq!(
            feed(
                &mut VietFlowEngine::new(options(false), CharsetId::Unicode),
                input
            ),
            regular
        );
        assert_eq!(
            feed(
                &mut VietFlowEngine::new(options(true), CharsetId::Unicode),
                input
            ),
            free
        );
    }
}
#[test]
fn eager_recovery_and_late_tone_respect_the_same_choice() {
    for input in ["cafp", "capf"] {
        let mut engine = VietFlowEngine::new(options(true), CharsetId::Unicode);
        assert_eq!(feed(&mut engine, input), "càp");
        let (remove, output) = engine.process_key(' ');
        assert_eq!((remove, output.as_str()), (0, " "));
    }
    let mut engine = VietFlowEngine::new(options(true), CharsetId::Unicode);
    assert_eq!(feed(&mut engine, "cafpe "), "càpe ");
}
#[test]
fn delimiter_case_method_and_output_charset_are_consistent() {
    for (method, input) in [
        (InputMethod::Telex, "capf"),
        (InputMethod::SimpleTelex, "capf"),
        (InputMethod::TelexSimple2, "capf"),
        (InputMethod::TelexVni, "capf"),
        (InputMethod::UserCustom, "capf"),
        (InputMethod::Vni, "cap2"),
        (InputMethod::Viqr, "cap`"),
    ] {
        for charset in [
            CharsetId::Unicode,
            CharsetId::UnicodeComposite,
            CharsetId::Tcvn3,
            CharsetId::VniWindows,
            CharsetId::Viqr,
        ] {
            for delimiter in [' ', '.', '!', '\n', '\t'] {
                if method == InputMethod::Viqr && delimiter == '.' {
                    continue;
                }
                let make = || {
                    VietFlowEngine::new(
                        EngineOptions {
                            method,
                            custom_keymap: CustomKeymap::telex(),
                            ..options(true)
                        },
                        charset,
                    )
                };
                let mut hook = make();
                let before = feed(&mut hook, input);
                let (remove, output) = hook.process_key(delimiter);
                assert_eq!(remove, 0, "{method:?}/{charset:?}");
                assert_eq!(output.as_str(), delimiter.to_string());
                let mut tsf = make();
                feed(&mut tsf, input);
                assert_eq!(tsf.finish_composition_on_break(delimiter), before);
            }
        }
    }
    assert_eq!(
        feed(
            &mut VietFlowEngine::new(options(true), CharsetId::Unicode),
            "CAPF "
        ),
        "CÀP "
    );
}
#[test]
fn english_list_wins_over_free_writing_for_all_detection_toggles() {
    for auto_switch_by_typing in [false, true] {
        for spell_check in [false, true] {
            for auto_non_vn_restore in [false, true] {
                let mut engine = VietFlowEngine::new(
                    EngineOptions {
                        auto_switch_by_typing,
                        spell_check,
                        auto_non_vn_restore,
                        custom_english_keywords: vec![
                            "of".into(),
                            "capf".into(),
                            "cafp".into(),
                            "test".into(),
                        ],
                        ..options(true)
                    },
                    CharsetId::Unicode,
                );
                assert_eq!(
                    feed(
                        &mut engine,
                        "of capf cafp test zoom zero file join texture future tieengs "
                    ),
                    "of capf cafp test zoom zero file join texture future tiếng "
                );
            }
        }
    }
}
#[test]
fn macro_undo_lookback_and_foreign_initial_rules_stay_available() {
    let mut engine = VietFlowEngine::new(
        EngineOptions {
            enable_macro: true,
            ..options(true)
        },
        CharsetId::Unicode,
    );
    engine.macros.insert("capf", "Nội dung riêng 😊");
    assert_eq!(feed(&mut engine, "capf "), "Nội dung riêng 😊 ");
    assert_eq!(
        feed(&mut engine, "of hello world capf "),
        "ò hello world Nội dung riêng 😊 "
    );
    assert_eq!(feed(&mut engine, "WWindows ofhf "), "Windows ofh ");
    let mut empty_initials = VietFlowEngine::new(
        EngineOptions {
            foreign_initials: vec![],
            ..options(true)
        },
        CharsetId::Unicode,
    );
    assert_eq!(feed(&mut empty_initials, "zoo "), "zoo ");
}
#[test]
fn c_abi_policy_change_resets_stale_composition_but_unchanged_refresh_does_not() {
    use vietflow_core::c_abi::*;
    unsafe {
        let handle = vf_engine_create();
        vf_engine_set_preserve_informal_marks(handle, true);
        let engine = handle as *mut VietFlowEngine;
        assert_eq!(feed(&mut *engine, "cap"), "cap");
        vf_engine_set_preserve_informal_marks(handle, true);
        assert_eq!((*engine).current_preedit_text(), "cap");
        let (remove, output) = (*engine).process_key('f');
        assert_eq!(remove, 2);
        assert_eq!(output.as_str(), "àp");
        assert_eq!((*engine).finish_composition_on_break(' '), "càp");
        feed(&mut *engine, "cap");
        vf_engine_set_preserve_informal_marks(handle, false);
        assert!((*engine).is_empty());
        assert_eq!(feed(&mut *engine, "capf "), "capf ");
        vf_engine_destroy(handle);
        vf_engine_set_preserve_informal_marks(std::ptr::null_mut(), true);
    }
}
