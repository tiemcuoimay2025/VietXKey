use vietflow_core::{CharsetId, CustomKeymap, EngineOptions, InputMethod, VietFlowEngine};

fn render(engine: &mut VietFlowEngine, input: &str) -> String {
    let mut screen = engine.current_preedit_text();
    for c in input.chars() {
        let (backspaces, output) = engine.process_key(c);
        for _ in 0..backspaces {
            assert!(screen.pop().is_some());
        }
        screen.push_str(output.as_str());
        assert_ne!(engine.take_auto_switch_request(), Some(false));
    }
    screen
}

#[test]
fn configured_z_accepts_accents_and_survives_space() {
    let mut engine = VietFlowEngine::default();
    assert_eq!(render(&mut engine, "zeef "), "zề ");
}

#[test]
fn foreign_initials_accept_all_telex_tones_and_capitalization() {
    for method in [
        InputMethod::Telex,
        InputMethod::SimpleTelex,
        InputMethod::TelexSimple2,
        InputMethod::TelexVni,
        InputMethod::UserCustom,
    ] {
        for auto in [false, true] {
            for free_marking in [false, true] {
                let mut engine = VietFlowEngine::new(
                    EngineOptions {
                        method,
                        auto_switch_by_typing: auto,
                        free_marking,
                        custom_keymap: CustomKeymap::telex(),
                        ..Default::default()
                    },
                    CharsetId::Unicode,
                );
                assert_eq!(
                    render(&mut engine, "zees zeef zeer zeex zeej ZeeF ZEEF "),
                    "zế zề zể zễ zệ Zề ZỀ ",
                    "{method:?}"
                );
                assert_eq!(render(&mut engine, "jaanf fowr "), "jần fở ", "{method:?}");
            }
        }
    }
}

#[test]
fn vni_and_viqr_support_the_same_foreign_initial() {
    for (method, input) in [
        (InputMethod::Vni, "ze62 "),
        (InputMethod::TelexVni, "ze62 "),
        (InputMethod::Viqr, "ze^` "),
    ] {
        let mut engine = VietFlowEngine::new(
            EngineOptions {
                method,
                ..Default::default()
            },
            CharsetId::Unicode,
        );
        assert_eq!(render(&mut engine, input), "zề ", "{method:?}");
    }
}

#[test]
fn configured_list_allows_extras_but_never_disables_normal_initials() {
    for (initials, expected) in [
        (vec![], "zeef joomf foos "),
        (vec!['Z'], "zề joomf foos "),
        (vec!['f', 'j', 'z'], "zề jồm fố "),
    ] {
        let mut engine = VietFlowEngine::new(
            EngineOptions {
                foreign_initials: initials,
                auto_switch_by_typing: false,
                ..Default::default()
            },
            CharsetId::Unicode,
        );
        assert_eq!(render(&mut engine, "zeef joomf foos "), expected);
        assert_eq!(render(&mut engine, "DDaf Nawngx "), "Đà Nẵng ");
    }
    let mut engine = VietFlowEngine::default();
    assert_eq!(
        render(&mut engine, "zbeef "),
        "zbeef ",
        "multi-consonant clusters remain invalid"
    );
}

#[test]
fn punctuation_backspace_and_english_words_do_not_undo_foreign_accents() {
    let mut engine = VietFlowEngine::default();
    assert_eq!(
        render(&mut engine, "zeef, ZeeF! zeef\x08eef "),
        "zề, Zề! zề "
    );
    assert_eq!(
        render(
            &mut engine,
            "zeef tieengs Vieetj Windows zero zoom file join "
        ),
        "zề tiếng Việt Windows zero zoom file join "
    );
    assert_eq!(
        render(&mut engine, "WWindows DDaf Nawngx "),
        "Windows Đà Nẵng "
    );
    assert_eq!(render(&mut engine, "zero"), "zero");
}

#[test]
fn tsf_and_legacy_charset_commits_preserve_foreign_accents() {
    for charset in [
        CharsetId::Unicode,
        CharsetId::UnicodeComposite,
        CharsetId::Tcvn3,
        CharsetId::VniWindows,
        CharsetId::Viqr,
    ] {
        let mut reference = VietFlowEngine::new(EngineOptions::default(), charset);
        let expected = render(&mut reference, "zeef");
        for delimiter in [' ', '.', '\n', '\t'] {
            let mut engine = VietFlowEngine::new(EngineOptions::default(), charset);
            let before = render(&mut engine, "zeef");
            assert_eq!(before, expected);
            let (backspaces, output) = engine.process_key(delimiter);
            assert_eq!(backspaces, 0, "{charset:?}");
            assert_eq!(output.as_str(), delimiter.to_string());
            let mut tsf = VietFlowEngine::new(EngineOptions::default(), charset);
            for c in "zeef".chars() {
                let _ = tsf.process_key(c);
            }
            assert_eq!(
                tsf.finish_composition_on_break(delimiter),
                expected,
                "{charset:?}"
            );
            assert!(tsf.is_empty());
        }
    }
}

#[test]
fn lone_w_shortcuts_and_simple_telex_w_initials_keep_their_meaning() {
    let mut engine = VietFlowEngine::default();
    assert_eq!(render(&mut engine, "w"), "ư");
    engine.reset();
    assert_eq!(render(&mut engine, "w\x08WWindows "), "Windows ");
    for method in [InputMethod::SimpleTelex, InputMethod::TelexSimple2] {
        let mut simple = VietFlowEngine::new(
            EngineOptions {
                method,
                ..Default::default()
            },
            CharsetId::Unicode,
        );
        assert_eq!(render(&mut simple, "weef "), "wề ");
    }
}

#[test]
fn native_ffi_applies_custom_mask_and_resets_only_on_change() {
    use vietflow_core::c_abi::*;
    use vietflow_core::english::foreign_initials_mask;
    unsafe {
        let handle = vf_engine_create();
        let engine = handle as *mut VietFlowEngine;
        vf_engine_set_foreign_initials_mask(handle, foreign_initials_mask(&['Z', 'z', '?']));
        assert_eq!(render(&mut *engine, "zee"), "zê");
        vf_engine_set_foreign_initials_mask(handle, 1 << 25);
        assert_eq!(
            (*engine).current_preedit_text(),
            "zê",
            "unchanged config must retain composition"
        );
        assert_eq!(render(&mut *engine, "f "), "zề ");
        vf_engine_set_foreign_initials_mask(handle, 0);
        assert_eq!(render(&mut *engine, "zeef "), "zeef ");
        vf_engine_set_foreign_initials_mask(handle, 0x8000_0000 | (1 << 25));
        assert_eq!(render(&mut *engine, "zeef "), "zề "); // unknown high bits ignored
        vf_engine_set_foreign_initials_mask(handle, 0);
        let _ = render(&mut *engine, "ze");
        vf_engine_set_foreign_initials_mask(handle, 1 << 25);
        assert!(
            (*engine).is_empty(),
            "changing config invalidates stale composition"
        );
        vf_engine_set_foreign_initials_mask(std::ptr::null_mut(), 0);
        vf_engine_destroy(handle);
    }
}
