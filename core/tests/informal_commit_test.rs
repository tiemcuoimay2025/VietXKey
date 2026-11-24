use vietflow_core::{CharsetId, CustomKeymap, EngineOptions, InputMethod, VietFlowEngine};
fn feed(engine: &mut VietFlowEngine, input: &str) -> String {
    let mut text = String::new();
    for c in input.chars() {
        let (remove, output) = engine.process_key(c);
        for _ in 0..remove {
            assert!(text.pop().is_some());
        }
        text.push_str(output.as_str());
    }
    text
}
#[test]
fn informal_oh_keeps_visible_tone_at_space() {
    let mut engine = VietFlowEngine::default();
    assert_eq!(feed(&mut engine, "ofh"), "òh");
    let (remove, output) = engine.process_key(' ');
    assert_eq!((remove, output.as_str()), (0, " "));
}
#[test]
fn configured_foreign_zo_keeps_visible_circumflex_at_space() {
    let mut engine = VietFlowEngine::default();
    assert_eq!(feed(&mut engine, "zoo"), "zô");
    let (remove, output) = engine.process_key(' ');
    assert_eq!((remove, output.as_str()), (0, " "));
}

#[test]
fn informal_words_survive_every_delimiter_method_option_and_case() {
    for method in [
        InputMethod::Telex,
        InputMethod::SimpleTelex,
        InputMethod::TelexSimple2,
        InputMethod::TelexVni,
        InputMethod::UserCustom,
    ] {
        for auto_switch_by_typing in [false, true] {
            for spell_check in [false, true] {
                for delimiter in [' ', '.', ',', '!', '?', '\n', '\t'] {
                    for (input, expected) in [
                        ("ofh", "òh"),
                        ("ohf", "òh"),
                        ("Ofh", "Òh"),
                        ("OFH", "ÒH"),
                        ("zoo", "zô"),
                        ("ZOO", "ZÔ"),
                        ("zoos", "zố"),
                        ("zoof", "zồ"),
                        ("afh", "àh"),
                        ("ahf", "àh"),
                        ("uwfh", "ừh"),
                        ("uwhf", "ừh"),
                        ("ofhhh", "òhhh"),
                        ("ohhhf", "òhhh"),
                    ] {
                        let mut engine = VietFlowEngine::new(
                            EngineOptions {
                                method,
                                auto_switch_by_typing,
                                spell_check,
                                custom_keymap: CustomKeymap::telex(),
                                ..Default::default()
                            },
                            CharsetId::Unicode,
                        );
                        assert_eq!(
                            feed(&mut engine, &format!("{input}{delimiter}")),
                            format!("{expected}{delimiter}"),
                            "{method:?}/{input}"
                        );
                        assert_ne!(engine.take_auto_switch_request(), Some(false));
                    }
                }
            }
        }
    }
}

#[test]
fn hook_and_tsf_share_current_word_recovery_in_all_charsets() {
    for charset in [
        CharsetId::Unicode,
        CharsetId::UnicodeComposite,
        CharsetId::Tcvn3,
        CharsetId::VniWindows,
        CharsetId::Viqr,
    ] {
        for input in [
            "ofh", "ohf", "zoo", "zoos", "zoof", "hix", "test", "tests", "mixx", "boxx", "loss",
            "WWindows", "DDaf", "zero", "zoom", "fool", "file", "join", "ash",
        ] {
            for delimiter in [' ', '.', '\n', '\t'] {
                let mut hook = VietFlowEngine::new(EngineOptions::default(), charset);
                let hooked = feed(&mut hook, &format!("{input}{delimiter}"));
                let mut tsf = VietFlowEngine::new(EngineOptions::default(), charset);
                let _ = feed(&mut tsf, input);
                let committed = tsf.finish_composition_on_break(delimiter);
                assert_eq!(
                    format!("{committed}{delimiter}"),
                    hooked,
                    "{charset:?}/{input}"
                );
                assert!(tsf.is_empty());
            }
        }
    }
    for (input, expected) in [
        ("hix", "hix"),
        ("test", "test"),
        ("tests", "tests"),
        ("mixx", "mix"),
    ] {
        let mut engine = VietFlowEngine::default();
        let _ = feed(&mut engine, input);
        assert_eq!(engine.finish_composition_on_break(' '), expected);
    }
}

#[test]
fn english_keywords_and_foreign_initial_configuration_keep_their_priority() {
    let mut engine = VietFlowEngine::default();
    assert_eq!(
        feed(
            &mut engine,
            "ofh zoo zero zoom Windows file join fool ash texture pure tieengs Vieetj "
        ),
        "òh zô zero zoom Windows file join fool ash texture pure tiếng Việt "
    );
    for method in [
        InputMethod::Telex,
        InputMethod::TelexVni,
        InputMethod::UserCustom,
    ] {
        let mut engine = VietFlowEngine::new(
            EngineOptions {
                method,
                custom_keymap: CustomKeymap::telex(),
                custom_english_keywords: vec!["ofh".into(), "ohf".into(), "zoo".into()],
                ..Default::default()
            },
            CharsetId::Unicode,
        );
        assert_eq!(
            feed(&mut engine, "ofh ohf zoo tieengs "),
            "ofh ohf zoo tiếng "
        );
    }
    let mut engine = VietFlowEngine::new(
        EngineOptions {
            foreign_initials: vec![],
            ..Default::default()
        },
        CharsetId::Unicode,
    );
    assert_eq!(feed(&mut engine, "zoo "), "zoo ");
}

#[test]
fn vni_and_viqr_commit_the_same_informal_spelling() {
    for (method, input) in [
        (InputMethod::Vni, "o2h oh2 zo6 u72h"),
        (InputMethod::Viqr, "o`h oh` zo^ u+`h"),
    ] {
        let mut engine = VietFlowEngine::new(
            EngineOptions {
                method,
                ..Default::default()
            },
            CharsetId::Unicode,
        );
        assert_eq!(feed(&mut engine, &format!("{input} ")), "òh òh zô ừh ");
    }
}

#[test]
fn informal_words_are_safe_lookback_boundaries_and_recover_after_backspace() {
    let mut engine = VietFlowEngine::default();
    assert_eq!(
        feed(
            &mut engine,
            "ofh makes sense zoo makes sense tieengs Vieetj "
        ),
        "òh makes sense zô makes sense tiếng Việt "
    );
    let mut edited = VietFlowEngine::default();
    assert_eq!(feed(&mut edited, "ofh\x08h zoo\x08oo "), "òh zô ");
    let mut undo = VietFlowEngine::default();
    assert_eq!(feed(&mut undo, "ofhf tieengs "), "ofh tiếng ");
}

#[test]
fn tsf_commit_respects_macro_priority_and_never_rewrites_previous_commits() {
    let mut options = EngineOptions::default();
    options.enable_macro = true;
    let mut engine = VietFlowEngine::new(options, CharsetId::Unicode);
    let replacement = "Nội dung 😊 ".repeat(40);
    engine.macros.insert("ofh", &replacement);
    let _ = feed(&mut engine, "ofh");
    assert_eq!(engine.finish_composition_on_break(' '), replacement);
    let mut tsf = VietFlowEngine::default();
    let mut committed = Vec::new();
    for word in [
        "ofh", "makes", "sense", "zoo", "makes", "sense", "test", "xong", "rooif",
    ] {
        let _ = feed(&mut tsf, word);
        committed.push(tsf.finish_composition_on_break(' '));
    }
    assert_eq!(
        committed.join(" "),
        "òh makes sense zô makes sense test xong rồi"
    );
}
