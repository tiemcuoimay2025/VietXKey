# VietXKey identity and theme update

The raised moss-green V keycap is an original SVG in `assets/VietXKey.svg`.
`scripts/generate-brand-assets.py` exports PNG and multi-resolution ICO files for
the Windows executable, TSF fallback icon and website. The native UI draws a
matching keycap directly using ImGui primitives. Existing tray V/E styles remain
available as user choices.

The reference https://gau.io.vn/ informed the soft radial light/dark background
and dimensional key motif. No Gau logos, images, CSS or wording were copied.
VietXKey retains Nunito typography, its own layout and moss accents.

`dark_theme` is persisted by both the Rust and C++ configuration readers. Missing
fields in old configurations default to dark. The header action switches on every
settings page and applies on the following frame. Resetting ImGuiStyle before
DPI scaling prevents repeated switches from accumulating scale. The Windows title
bar follows the saved theme. The website separately follows system preference,
then persists the visitor's explicit theme in localStorage.

Validation on 2026-11-24:

- Rust workspace: 232 passed, 3 upstream tests ignored by default.
- Native configuration persistence CTest passed, including legacy defaults and
  light/dark save/reload.
- Offscreen production UI rendered all seven pages in both themes. Actual header
  theme actions applied, persisted and restored; existing startup/language/method,
  free-writing and QR modal interaction checks passed.
- Native UI library built for x64 and x86. The downloadable executable and TSF
  DLL are x64. No ARM64 build or new MSI package is claimed.

The offscreen renderer uses real production UI, fonts and DX11/WARP but fixture
TSF and sound callbacks. It does not register a keyboard service or validate
keystrokes inside third-party apps. Earlier Comet fixes were confirmed by the user.

## Website verification

The standalone site in `site/` was exercised with an isolated Edge Chromium
profile at 320, 390, 820 and 1440 px, in light and dark themes. Checks passed for
no horizontal overflow, loaded images, no console/page/request errors, saved
explicit theme, system-theme changes, screenshot modal, support QR modal,
Escape / close / backdrop dismissal, FAQ, local anchors and exact ZIP bytes.
The website has no analytics or remotely loaded fonts. Its theme choice is
separate from the desktop app's preference.
