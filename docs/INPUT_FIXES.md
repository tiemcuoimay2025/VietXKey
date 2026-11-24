# VietXKey input fixes

## Modifier shortcut overlap

Previously Ctrl+Shift toggled on the second modifier's keydown. That meant an
application shortcut such as Ctrl+Shift+X had already changed the input mode
before X arrived. Modifier-only gestures now toggle on release, once, only if
the gesture stayed clean. Extra letters, function/editing keys or modifiers
cancel the pending toggle until the modifiers are released. Ctrl/Shift key
events themselves are passed through to the foreground application.

The same guard covers Alt+Shift and custom modifier-only combinations. Keyed
shortcuts require an exact modifier set. Physical key state and an event counter
are recorded before trying the hook mutex: skipped events cancel a pending
gesture rather than accidentally completing one. The current modifier event is
overlaid on GetAsyncKeyState because the low-level callback runs before Windows
updates that state. Focus changes discard incomplete gestures.

## Mixed English and Vietnamese

English restoration previously emitted a global mode-off request, including
mid-word tone-key undo. The hook then toggled the persistent V/E selection and
reset the engine, making the next Vietnamese word depend on English-mode
lookback detection. Restoration now affects only the current word. Vietnamese
stays enabled and the following word can compose accents immediately.

English-mode detection of Vietnamese text is retained. Manual V/E changes,
application exclusions and intentional game mode behaviour are preserved.

Examples exercised through the engine:

- `was tieengs Vieetj ` -> `was tiếng Việt `
- `this makes sense xin chaof ` -> `this makes sense xin chào `
- `mixx tieengs Vieetj ` -> `mix tiếng Việt `
- Custom English keyword `props` followed by `xong rooif` -> `props xong rồi`
- VNI and Telex+VNI: `was tie6ng1 Vie6t5 ` -> `was tiếng Việt `
- Invalid words, punctuation, macros and repeated language alternation.

## Validation and Windows-managed input

### Chromium/Comet inline autocomplete (DĐà)

The user's Comet address bar displayed `DĐà` when typing `DDaf`. Read-only
inspection initially found an older DLL still mapped in Comet, but the user
fully restarted Comet and reproduced `DĐà` again. The reopened browser was
using the hook path, not an active VietXKey TSF composition. Stale DLL loading
therefore did not explain the whole problem.

Chromium's omnibox reverse-selects its inline completion: anchor at the end of
the suggestion, caret at the end of the user's actual text. The former
Backspace+Unicode transaction deleted the selected suggestion, left the typed
`D` behind, then inserted `Đ`. A deterministic native input-event model
reproduces exactly `DĐ` with the original production batch builder.

The first Shift+Left/Unicode-only implementation passed a static-selection
model but failed the user's direct tests: both an inline URL completion and
old word suffixes (`ôngo`, `ông hoàngang`) remained in Comet, including page
inputs. It relied on Unicode delivery replacing a still-live selection.

Auto mode now extends the selection across the old characters, explicitly
deletes it with Backspace, and only then emits Unicode. It does not depend on
selection surviving Unicode/IME delivery. The Left events carry their E0
extended-key identity. Without completion only the old characters are selected
and deleted, including editing in the middle of text. A physically held Shift
is preserved; synthetic modifiers are balanced. Enter/Tab remain last in the
same checked SendInput call; partial acceptance never replays edits.

The route is restricted to Auto mode, Unicode charset, ordinary bounded text
rewrites and exact browser process names (Comet, Chrome, Chromium, Edge, Brave,
Vivaldi, Opera). Other applications, explicit injection modes, composite/legacy
charsets and larger/paced macros retain the existing route. TSF is unchanged.
The new delivery route is internal and does not add a persisted injection mode.

Validation: **207 Rust tests passed, 3 existing tests ignored**. Nine browser tests
cover the before/after `DĐ` failure, full `DDaf Nawngx` sequences, mixed typing
including Windows/zề, no completion, held Shift, existing surrounding text,
deletion, Unicode surrogate pairs, word-break ordering, bounded payloads,
dispatch scope and partial SendInput cleanup. The added defensive host model
collapses selection before Unicode and reproduces the reported retained
`o`, `ang` and URL tails with deletion omitted; the explicit-delete sequence
passes that model as well as the original one. This is not an exact model of
Comet's internal event loop. Native-app automation failed to initialize with
sandbox error 1920, so the assistant did not automate typing in Comet.

**Direct user validation, 2026-11-22:** after installing the explicit-delete
EXE, the user confirmed both the page chat input and address-bar test sequences
were correct. Their screenshots show `ông hoàng` in the chat field and `Đà`
in the omnibox, with no retained word/URL suffix. A read-only checksum of the
installed EXE matched the tested build:
`5b0b62eb43ccf7df921968dc69cf3ab05255e57538d56d4a5a87e9da2376a625`.
This confirmation covers the user's Comet setup; other Chromium browsers are
covered by process routing and automated regression tests, not manual trials.

Primary-source reference for the reverse selection:
https://raw.githubusercontent.com/chromium/chromium/main/chrome/browser/ui/views/omnibox/omnibox_view_views.cc
(`OnInlineAutocompleteTextMaybeChanged`, selection is display length → user-text length).

### Accented foreign initials (zề)

The foreign-initial setting previously acted as a denylist: a configured `z`
blocked both tone and vowel modification. Even removing the block alone was
insufficient, because spell checking rejected `z` as an initial at commit.
The regression `zeef ` -> `zeef ` was reproduced before this change.

The list now permits additional single consonant initials. Default `fjwz`
allows `zeef` -> `zề`, including capitalized forms and all tone keys. With spell
checking enabled, clearing the list disables extra initials; missing JSON
still receives default `fjwz`.
Standard Vietnamese initials are always accepted independently of this list.
Vowel/coda validation and English keyword protection still apply. Known
English words such as `zero`, `zoom`, `file` and `join` are preserved. Telex's
lone-W shortcut and explicit WW escape retain their existing meaning; in
Simple Telex methods, plain W can be a configured consonant initial.

The tooltip now explains the permitted-initial behavior and shows `zeef → zề`.
Hook mode already consumes the normalized list. TSF now receives the same
list as a bounded A-Z bitmask and applies it through an additive C ABI setter.
The configuration mapping is versioned as `VietFlow_TSF_Config_v3`; its 48-byte
layout reuses the old reserved word for the mask. The focus protocol remains
v2. Both EXE and DLL must be updated, and applications hosting an older TIP
must be reopened. The new reader rejects v2 snapshots and invalid mask bits.

Validation: **198 Rust tests passed, 3 existing tests ignored**. Added coverage
includes all tone keys, upper/lowercase, Telex variants/custom Telex, VNI/VIQR,
all five charsets, empty/custom config, backspace, punctuation, English mixing,
TSF finalization and C ABI configuration changes. Native IPC, cross-process
mask publication, and the release-core C++ consumer are checked separately.
These are automated checks; no manual typing into the user's apps is claimed.

### Literal escapes and capitalized Vietnamese (Windows / Đà Nẵng)

Two additional failures were reproduced before the fix:

- `WWindows ` displayed `Windows` until Space, then became `WWindows `.
  The second W intentionally undid lone-W composition (`Ư` -> `W`), but
  automatic restoration later replayed both physical keys.
- `DDaf Nawngx ` displayed `Đà`, then changed it to `DDaf` on Space.
  The acronym/identifier heuristic counted the two uppercase action keys
  rather than the single uppercase letter `Đ` in the visible word.

Explicit modification undo is now preserved until the word ends, including
WW, DDD, circumflex/horn undo and VNI undo. It is distinct from automatic
English restoration and acts as a lookback boundary. Macros retain their
existing priority. The flag resets at word/reset boundaries; the next word
can still compose Vietnamese. Both hook-delivered text and TSF finalization
respect the visible result instead of replaying the escape key history.

Capitalization decisions now count composed Unicode letters, and frequency
normalization lowercases Vietnamese Unicode as well as ASCII. `DDaf`, `Ddaf`,
and `DDAF` therefore retain their intended Vietnamese text, while programming
identifiers and English acronyms keep their existing behavior.

The native hook also consumes a text action whose optimized delta is empty
(e.g. `az` -> `a`); it no longer leaks that action as a literal character.

Regression coverage includes before/after word-break assertions, capitalization,
backspace/retyping, punctuation/Enter/Tab, mixed English/Vietnamese, option
combinations, custom Telex, VNI, all five output charsets and TSF preedit commit.
`cargo test --workspace`: **188 passed, 3 existing environment-dependent tests
ignored**. These are automated engine/native-delivery tests; no manual typing
into the user's applications is claimed.

The original mixed-word regression was reproduced before the fix. That earlier
milestone passed 175 tests (3 upstream environment-dependent tests ignored).
The new shortcut tests cover both modifier orders, both release orders,
Ctrl+Shift+X/F5, editing keys, held/repeated keys, extra modifiers, skipped
events, focus changes and custom aliases. Production native binaries are rebuilt
with the updated core. Desktop end-to-end typing has not been automated.

This fixes VietXKey's own hotkey handling. In registered TSF mode the existing
architecture delegates keyboard selection to Windows. Windows itself also uses
Ctrl+Shift for switching keyboard layouts. If the Windows ENG/VIE indicator
changes, that system shortcut is separate from this application fix; Win+Space
can select the intended Windows keyboard. No Windows language settings were
changed by this customization.

Microsoft references:
- https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
- https://support.microsoft.com/en-gb/accessibility/windows/keyboard-shortcuts-in-windows?nochrome=true


### Informal accents at commit (òh / zô), 2026-11-23

Reproduced before the fix with the default configuration:

| Input | Before Space | After Space (old) | New commit |
| --- | --- | --- | --- |
| `ofh` | òh | ofh | òh |
| `zoo` | zô | zoo in Hook; zô in TSF | zô in both |
| `ohf` | ohf | ohf | òh |

The spelling fallback rejected the informal final h, while the English corpus
preferred the English word zoo over a permitted foreign-initial composition.
The fix preserves a toned single-vowel interjection followed by h (including
repeated h) unless its raw keys match an English keyword/corpus word. It also
preserves a marked, phonologically valid word with a configured foreign initial,
ahead of the statistical English decision. Built-in/custom English keywords
retain priority, so zero/zoom/file/join and a user-added zoo still remain English.
This does not globally disable spelling or English restoration. The unrelated
invalid-coda/vowel checks and explicit double-key undo retain their behavior.

Tone placement now handles both orders (ofh and ohf), including VNI/VIQR.
An intentional informal word clears lookback so a following English word cannot
later replace it with the physical tone keys.

The audit also reproduced current-word commit drift: Hook preserved test/hix,
while TSF returned tét/hĩ; a doubled tone-key recovery such as mixx differed as
well. Both paths now use one finalization function for macros, undo, slang,
English recovery and the new informal spellings. TSF applies only the current
word delta and never requests edits to already committed preceding words.
Hook's existing multiword lookback remains available in Hook mode.

Validation: 222 Rust workspace tests passed, with 3 existing environment-dependent
tests ignored. Eight new regressions cover pre/post-commit text, Telex variants
and custom Telex, VNI/VIQR, five charsets, capitals, options, delimiters,
backspace/retyping, undo, English/custom-keyword priority, foreign-initial
settings, long emoji-containing macros, Hook/TSF parity and following words.
A separately built C++ consumer of the release Rust library passed the same
owned UTF-8 functions used by TSF, including ofh/ohf/zoo/zoos/test/hix/mixx.
No live typing into the user's applications or TSF registration was performed.

For an ambiguous word intentionally kept in English, use the existing
**Từ tiếng Anh giữ nguyên** list. The optional broader free-writing preference is described below; the targeted
òh/zô fixes still work when that preference is off.


### Optional free-writing accent preservation, 2026-11-23

**Gõ tiếng Việt → Dấu tiếng Việt → Giữ dấu khi viết không chuẩn chính tả**
is opt-in (JSON/core default false), independently of the existing free-marking
option. Missing fields in old configurations stay false. Both Rust and C++ save
and reload `preserve_informal_marks` without changing other preferences.

When enabled, existing Vietnamese marks survive eager spelling fallback and
word commit, even for an invalid syllable such as càp. Both `cafp` and `capf`
produce càp. Stop-coda and coda spelling restrictions on tone insertion are
relaxed for this option. Explicit undo, macros and configured foreign-initial
permissions still apply. This preference preserves a visible spelling; it does
not automatically correct typos or guarantee that arbitrary multiword strings
can be composed as one Vietnamese syllable. Ambiguous English such as of/test
can become ò/tét. English-keyword checks in the engine retain priority.

The new additive C ABI setter resets composition only when the value changes.
Hook copies the flag from JSON to EngineOptions. TSF receives it as a validated
u32 in the v4, 52-byte configuration snapshot (`VietFlow_TSF_Config_v4`); focus
remains v2. v3 snapshots and malformed flag values are rejected. Update both
EXE and DLL and reopen host applications after updating. The TSF protocol carries
this flag and the existing built-in engine rules; it does not introduce transport
of the GUI's custom/removed English keyword lists, an existing TSF limitation.

A related regression found during the option tests was fixed: spell recovery
had recorded an intermediate preedit in lookback even after committing raw
English. A following word could therefore duplicate a prefix (test zoom →
ttest zoom). Lookback now records the actual final text and matching length.

Validation: 231 Rust tests passed (3 pre-existing environment-dependent tests
ignored), including on/off behavior, eager recovery, late tones, methods,
charsets, delimiter handling, English-list priority in the engine, macros,
undo, stale-composition reset and the committed-text lookback regression.
C++ config persistence passed; native layout/IPC smoke and cross-process IPC
stress passed; the release-core C++ FFI consumer followed the flag at TSF commit.
The isolated native UI renderer verified checkbox default and persisted on/off
and was inspected at 100% and 150% DPI. No keyboard hook or TSF registration was
installed by QA, and typing in the user's applications was not automated.
