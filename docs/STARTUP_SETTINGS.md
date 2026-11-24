# Settings visibility on manual launch, elevation and Windows login

A manual launch, including the very first launch, opens the control panel after
initializing the daemon and tray. An explicit restart with administrator rights
also opens it, independently of the Windows-login display preference. Opening
an already-running application requests the running daemon's settings window.

Only Windows login (`--startup`) uses both `start_with_windows` and
`show_settings_on_startup`. The second option defaults to true in Rust and C++;
an explicit false keeps login startup in the tray. Turning off the master
startup option retains the display preference. The Run entry quotes the EXE
path and appends `--startup`; running the updated EXE refreshes an older entry.

The UI stays in a separate `--settings` process. UI/converter children do not
spawn a daemon or another UI. A newly created window explicitly restores itself
after the first ShowWindow call so inherited hidden/minimized startup state
cannot keep the panel invisible. User-initiated requests also pass foreground
permission from the launcher/UI to the daemon and then its GUI child.

The elevation request carries `--elevated --config "<same config path>"`. The
existing UI remains available if the user cancels UAC. On success, the old daemon
closes its UI children and exits. The elevated process waits on the same named
mutex for up to ten seconds, accepting both normal release and an abandoned
mutex after owner exit, before installing its hook and opening a fresh panel.
The ownership guard releases and closes the mutex on shutdown. A second daemon
never starts while the original still owns the mutex.

Validation: 214 Rust workspace tests passed (3 environment-dependent tests
ignored). Regressions cover all combinations of login options, manual/admin
launches, UI child aliases, Unicode config paths, exclusive handoff and an owner
thread exiting without releasing its mutex. C++ configuration tests verify
startup defaults and explicit opt-out persistence. Native offscreen UI tests
verify the startup switches. Tests do not request UAC elevation or sign out the
user; the policy and mutex handoff are exercised in isolated fixtures.
