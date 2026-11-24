# VietXKey — moss UI customization

Local customization of VietFlow v0.1.4 for Windows. The design takes inspiration
from the supplied reference's rounded typography and sidebar, with its own
branding, copy, navigation groups, card proportions and moss palette.

The product is branded **VietXKey** in the sidebar, About page, window titles,
tray text, Windows executable metadata and Windows keyboard profile labels.
The x64 portable distribution uses `VietXKey.exe`. About identifies the original
author as VietFlow Team and the customization as
"Fix lỗi và phát triển thêm bởi Minh Tâm Đoàn." It also credits the UniKey
algorithm. The requested upstream update menu item, repository/release buttons,
repository URL in the native About dialog and upstream donation section are removed.
The About page now has **Ủng hộ tác giả**, which opens a modal showing the exact
user-supplied QR for Minh Tâm Đoàn. The PNG is embedded in the executable, with
nearest-neighbor image sampling and its original white margin. It requires no
external QR file or network connection. Device minimize/restore releases and
recreates the QR texture. Escape, the title-bar close button and Đóng dismiss it.
Internal ABI and IPC identifiers retain their existing names for compatibility.
Generated files use `vietxkey_config.json`, `vietxkey_config.json.bak`,
`vietxkey_tsf.dll` and (when diagnostics are enabled) `vietxkey_tsf.log`.
Shared configuration folders use `VietXKey`. Both hosts migrate valid legacy
configuration bytes, preserving macros, profiles, hotkeys, unknown fields and
the old backup. An existing valid new primary or backup always takes precedence.
Original legacy configuration files are kept for recovery/coexisting installs. This distribution does not include a rebranded MSI.

## Visual direction

- Background: neutral charcoal `#131314`.
- Sidebar: `#1B1B1E` to `#17171A`.
- Hero: charcoal gray `#202024` to `#36363B`.
- Primary actions: `#4F663B`; accent: `#BACF85`.
- Body: embedded Nunito Medium, 20 logical pixels (increased from 16 px).
- Captions and navigation groups: at least 14 logical pixels; descriptions: 15–17.5 px.
- Headings: embedded Nunito ExtraBold; 32.5 px page title and 40 px overview hero title.
- Cards: 16 px corners; hero: 18 px corners. Sizes scale with monitor DPI.

The overview combines method and charset selectors into a wider card, followed
by live macro and excluded-app counts. Two lower cards group daily settings
and language shortcuts. The previous overview shortcut controls are available
on the new dedicated Shortcuts page. Other feature pages keep their existing
controls under the shared theme and navigation.

Navigation and Save remain fixed while each page has its own scrolling region.
The Windows-managed language state disables the sidebar language switch and
shows the system shortcut. Restoring defaults opens a confirmation dialog.
The sidebar is wider to accommodate larger labels. Its navigation area scrolls
independently on short windows so the language switch and footer stay available.
Controls and cards have extra height for the larger text. The About page retains
the Minh Tâm Đoàn customization credit below its subtitle.

## Build

Use the upstream build instructions. No additional runtime dependency or font
installation is needed. The supplied portable build is Windows x64 with static
CRT, using the existing ImGui/DX11 frontend and Rust input engine.

Build the branded x64 portable files with `scripts/build-vietxkey.ps1`; outputs
are written to `dist/VietXKey-x64/`. The upstream build script remains available
for developer builds of the original Cargo target.

## Validation

- Release C++ UI build and Windows x64 executable build succeeded.
- Rust workspace: 232 tests passed; 3 upstream tests were ignored by default.
- C++ configuration persistence and legacy filename migration tests passed.
- Release input selection checks passed for synthetic x64/x86/ARM64 files.
- Full offline release-payload regressions passed in an isolated local copy
  without the upstream tag mismatch; no GitHub publishing occurred.
- Rust migration regression covers primary/backup precedence, invalid source
  recovery, preservation of unknown fields and unchanged legacy files.
- An isolated development renderer exercised the production UiManager, actual
  font assets, config persistence and DirectX 11 backend. Navigation snapshots
  were inspected for all seven pages, plus the overview at 1000 x 660, normal
  dimensions, and 150% DPI. A registered-TSF fixture was also rendered.
- QR input tests verified opening, Escape, Đóng and reopening. The original PNG,
  embedded bytes, decoded GPU texture and displayed 256x256 region were identical.
- UI input tests verified startup toggle persistence, language switching,
  the Windows-managed language lock, and persisted method selection.
- Visual QA used offscreen rendering because the desktop automation tool was
  unavailable. It did not register TSF or start a system keyboard hook.

The preview renderer's TSF state and sound callbacks are isolated fixtures;
snapshots validate UI presentation and interactions, not typing in other apps.
The Rust engine and host code were tested with the repository's workspace suite.

The project remains GPLv3; font licenses are included alongside the assets.

## Default macros

Core, Windows host and C++ settings defaults use `vxk` → `VietXKey`.
Loading a configuration upgrades only the exact legacy sample `vf` → `VietFlow`.
A custom `vf` or `vxk` mapping remains unchanged, and deleting the sample does not
cause it to be re-added. Regression tests cover upgrade, save/reload, custom
mappings and actual engine expansion on space and punctuation.

## Informal spelling commit

`ofh` and `ohf` commit as òh; `zoo` commits as zô with the configured foreign
initial z. Space, punctuation, Enter and Tab retain those accents. Current-word
commit now shares its policy between Hook and TSF. See `docs/INPUT_FIXES.md` for
reproductions, scope, English priority and native ABI validation.

## Optional free writing

The Typing page's Dấu tiếng Việt section includes **Giữ dấu khi viết không chuẩn
chính tả** with an explanatory tooltip. It defaults off and is distinct from
Đặt dấu tự do. Native UI QA toggled and reloaded both values successfully.
The setting is shared by Hook and TSF through the v4 configuration snapshot;
see `docs/INPUT_FIXES.md` for behavior and the existing TSF keyword-list limit.
