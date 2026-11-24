// Isolated Windows regression for C++ GUI configuration persistence.
// Opt-in via -DVIETFLOW_BUILD_CONFIG_TESTS=ON; never touches the live config.
#include "config.h"
#include <windows.h>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

namespace fs = std::filesystem;

static std::string bytes(const fs::path& path) {
    std::ifstream in(path, std::ios::binary);
    return {std::istreambuf_iterator<char>(in), std::istreambuf_iterator<char>()};
}

static bool check(bool condition, const char* message) {
    if (!condition) std::cerr << "FAIL: " << message << "\n";
    return condition;
}

static bool spawn_writer(const fs::path& path, int marker, PROCESS_INFORMATION& out) {
    wchar_t exe[MAX_PATH] = {};
    const DWORD length = GetModuleFileNameW(nullptr, exe, MAX_PATH);
    if (length == 0 || length >= MAX_PATH) return false;
    const std::wstring cmd = L"\"" + std::wstring(exe) +
        L"\" --stress-writer \"" + path.wstring() +
        L"\" " + std::to_wstring(marker);
    std::vector<wchar_t> args(cmd.begin(), cmd.end());
    args.push_back(L'\0');
    STARTUPINFOW startup = {};
    startup.cb = sizeof(startup);
    return CreateProcessW(
        exe, args.data(), nullptr, nullptr, FALSE, CREATE_NO_WINDOW,
        nullptr, nullptr, &startup, &out
    ) != FALSE;
}

int main(int argc, char** argv) {
    try {
        if (argc == 4 && std::string(argv[1]) == "--stress-writer") {
            const int marker = std::stoi(argv[3]);
            if (marker != 1 && marker != 2) return 80;
            for (int i = 0; i < 60; ++i) {
                vietflow::Config writer;
                writer.method = marker;
                writer.enabled = marker == 1;
                writer.macros["writer-marker"] = marker == 1 ? "writer-one" : "writer-two";
                if (!writer.Save(argv[2])) return 81;
            }
            return 0;
        }
        const auto directory = fs::temp_directory_path() /
            ("vietflow-ui-config-test-" + std::to_string(GetCurrentProcessId()) +
             "-" + std::to_string(GetTickCount64()));
        fs::create_directories(directory);
        struct Cleanup {
            fs::path root;
            ~Cleanup() { std::error_code ec; fs::remove_all(root, ec); }
        } cleanup{directory};

        const auto themePath = directory / "theme.json";
        { std::ofstream f(themePath); f << "{}"; }
        vietflow::Config theme;
        if (!check(theme.Load(themePath.string()) && theme.dark_theme, "legacy theme remains dark")) return 103;
        for (bool dark : {false, true}) {
            theme.dark_theme = dark;
            if (!check(theme.Save(themePath.string()), "save selected theme")) return 104;
            vietflow::Config loaded;
            if (!check(loaded.Load(themePath.string()) && loaded.dark_theme == dark, "theme survives save/reload")) return 105;
        }
        const auto writingPath = directory / "free-writing.json";
        { std::ofstream f(writingPath); f << "{}"; }
        vietflow::Config writing;
        if (!check(writing.Load(writingPath.string()) && !writing.preserve_informal_marks,
                   "legacy config defaults free writing off")) return 100;
        for (bool enabled : {true, false}) {
            writing.preserve_informal_marks = enabled;
            if (!check(writing.Save(writingPath.string()), "save free-writing preference")) return 101;
            vietflow::Config loaded;
            if (!check(loaded.Load(writingPath.string()) && loaded.preserve_informal_marks == enabled,
                       "free-writing preference persists")) return 102;
        }
        const auto macroPath = directory / "macro-upgrade.json";
        if (!check(vietflow::Config().macros.at("vxk") == "VietXKey" && vietflow::Config().macros.count("vf") == 0,
                   "branded default macro")) return 96;
        struct MacroCase { const char* json; const char* vf; const char* vxk; };
        for (const auto& fixture : {
            MacroCase{R"({"macros":{"vf":"VietFlow","mine":"keep"}})", nullptr, "VietXKey"},
            MacroCase{R"({"macros":{"vf":"custom","vxk":"my expansion","mine":"keep"}})", "custom", "my expansion"},
            MacroCase{R"({"macros":{"vf":"VietFlow","vxk":"my expansion","mine":"keep"}})", nullptr, "my expansion"},
            MacroCase{R"({"macros":{"mine":"keep"}})", nullptr, nullptr}
        }) {
            { std::ofstream f(macroPath); f << fixture.json; }
            vietflow::Config macroConfig;
            if (!check(macroConfig.Load(macroPath.string(), false), "load macro upgrade fixture")) return 97;
            auto matches = [&](const char* key, const char* value) {
                auto found = macroConfig.macros.find(key);
                return value ? found != macroConfig.macros.end() && found->second == value : found == macroConfig.macros.end();
            };
            if (!check(matches("vf", fixture.vf) && matches("vxk", fixture.vxk) && matches("mine", "keep"),
                       "upgrade exact sample while preserving custom/deleted mappings")) return 98;
            if (!check(macroConfig.Save(macroPath.string()) && macroConfig.Load(macroPath.string()) && matches("vxk", fixture.vxk),
                       "macro upgrade survives save/reload")) return 99;
        }

        // Real branded upgrade: raw fields and a distinct previous backup
        // survive; valid new primary/backup always wins over the old name.
        const auto branded = directory / "vietxkey_config.json";
        const auto legacy = directory / "vietflow_config.json";
        auto brandedBackup = branded; brandedBackup += L".bak";
        auto legacyBackup = legacy; legacyBackup += L".bak";
        const std::string oldRaw = R"({"method":1,"hotkey_mode":"AltZ","macros":{"brb":"Đà Nẵng"},"app_profiles":{"comet.exe":{"method":2}},"future_setting":{"keep":true}})";
        const std::string backupRaw = R"({"method":2,"macros":{"backup":"preserved"}})";
        { std::ofstream f(legacy); f << oldRaw; }
        { std::ofstream f(legacyBackup); f << backupRaw; }
        const std::vector<std::string> sources{legacy.string()};
        if (!check(vietflow::Config::MigrateLegacyConfig(branded.string(), sources), "migrate legacy name") ||
            !check(bytes(branded) == oldRaw && bytes(brandedBackup) == backupRaw, "raw config and distinct backup retained") ||
            !check(bytes(legacy) == oldRaw && bytes(legacyBackup) == backupRaw, "legacy originals untouched")) return 90;
        vietflow::Config migrated;
        if (!check(migrated.Load(branded.string()) && migrated.hotkey_mode == "AltZ" &&
                   migrated.macros["brb"] == "Đà Nẵng" && migrated.app_profiles["comet.exe"].method == 2,
                   "macros profiles and hotkeys survive")) return 91;
        { std::ofstream f(legacy); f << R"({"method":0})"; }
        if (!check(vietflow::Config::MigrateLegacyConfig(branded.string(), sources) && bytes(branded) == oldRaw,
                   "new primary takes precedence")) return 92;
        { std::ofstream f(branded); f << "broken-new-file"; }
        if (!check(vietflow::Config::MigrateLegacyConfig(branded.string(), sources) && bytes(branded) == "broken-new-file" &&
                   migrated.Load(branded.string()) && migrated.method == 2, "new backup takes precedence")) return 93;
        fs::remove(brandedBackup);
        { std::ofstream f(legacy); f << R"({"method":"invalid"})"; }
        if (!check(vietflow::Config::MigrateLegacyConfig(branded.string(), sources) && bytes(branded) == backupRaw,
                   "recover old backup when primary is invalid")) return 94;
        const auto absent = directory / "empty" / "vietxkey_config.json";
        { std::ofstream f(legacyBackup); f << "invalid-backup"; }
        if (!check(!vietflow::Config::MigrateLegacyConfig(absent.string(), sources) && !fs::exists(absent),
                   "invalid legacy data never creates defaults")) return 95;

        const auto primary = directory / "settings.json";
        auto backup = primary;
        backup += L".bak";

        // Old startup configurations inherit the new show-settings default;
        // an explicit user opt-out must survive GUI persistence.
        const auto startup_path = directory / "startup-settings.json";
        {
            std::ofstream legacy(startup_path);
            legacy << R"({"start_with_windows":true})";
        }
        vietflow::Config startup;
        if (!check(startup.Load(startup_path.string()) && startup.show_settings_on_startup,
                   "legacy startup config enables settings window")) return 72;
        startup.show_settings_on_startup = false;
        if (!check(startup.Save(startup_path.string()), "save startup UI opt-out")) return 73;
        vietflow::Config startup_reloaded;
        if (!check(startup_reloaded.Load(startup_path.string()) && !startup_reloaded.show_settings_on_startup,
                   "startup UI opt-out persists")) return 74;

        vietflow::Config original;
        original.enabled = true;
        original.macros["recovery-marker"] = "preserve-me";
        if (!check(original.Save(primary.string()), "initial save")) return 1;

        vietflow::Config updated;
        updated.enabled = false;
        updated.method = 2;
        if (!check(updated.Save(primary.string()), "second valid save")) return 2;

        vietflow::Config previous;
        if (!check(previous.Load(backup.string()), "load last valid backup") ||
            !check(previous.macros["recovery-marker"] == "preserve-me", "backup data")) return 3;
        const std::string backup_before = bytes(backup);

        // A JSON object may parse successfully but fail typed deserialization
        // after applying earlier fields. The saved backup should recover the
        // entire config, not leave a half-updated in-memory UI state.
        {
            std::ofstream malformed(primary, std::ios::binary | std::ios::trunc);
            malformed << R"({"enabled":false,"method":"not-a-number","macros":{}})";
        }
        vietflow::Config semantic_recovery;
        semantic_recovery.enabled = false;
        semantic_recovery.method = 6;
        semantic_recovery.macros.clear();
        semantic_recovery.macros["sentinel"] = "should-be-replaced-from-backup";
        if (!check(semantic_recovery.Load(primary.string()), "semantic invalid primary falls back to backup") ||
            !check(semantic_recovery.enabled && semantic_recovery.method == 0,
                   "fallback restores coherent backup fields") ||
            !check(semantic_recovery.macros.at("recovery-marker") == "preserve-me",
                   "fallback restores backup macros") ||
            !check(semantic_recovery.macros.count("sentinel") == 0, "no leaked partial values")) return 9;

        // If neither candidate is safe, Load must fail without changing ANY
        // values. Both files are valid JSON syntax but contain wrong field types.
        const auto invalid = directory / "invalid-settings.json";
        auto invalid_backup = invalid;
        invalid_backup += L".bak";
        {
            std::ofstream out(invalid, std::ios::binary | std::ios::trunc);
            out << R"({"enabled":false,"method":"not-a-number"})";
        }
        {
            std::ofstream out(invalid_backup, std::ios::binary | std::ios::trunc);
            out << R"({"enabled":false,"method":[]})";
        }
        vietflow::Config unchanged;
        unchanged.enabled = true;
        unchanged.method = 5;
        unchanged.macros["custom-sentinel"] = "must survive failure";
        if (!check(!unchanged.Load(invalid.string()), "both semantically invalid candidates rejected") ||
            !check(unchanged.enabled && unchanged.method == 5,
                   "failed Load preserves scalar fields") ||
            !check(unchanged.macros.at("custom-sentinel") == "must survive failure",
                   "failed Load preserves macros")) return 10;

        {
            std::ofstream broken(primary, std::ios::binary | std::ios::trunc);
            broken << "{corrupt";
        }
        if (!check(updated.Save(primary.string()), "recover after corrupted primary")) return 4;

        size_t snapshots = 0;
        for (const auto& entry : fs::directory_iterator(directory)) {
            const auto filename = entry.path().filename().string();
            if (filename.find("settings.json.corrupt.") == 0) {
                ++snapshots;
                if (!check(bytes(entry.path()) == "{corrupt", "quarantine preserves original bytes")) return 5;
            }
        }
        if (!check(snapshots == 1, "exactly one quarantine snapshot")) return 6;
        if (!check(bytes(backup) == backup_before, "good backup never overwritten by corruption")) return 7;

        vietflow::Config recovered;
        if (!check(recovered.Load(primary.string()), "load recovered primary") ||
            !check(!recovered.enabled && recovered.method == 2, "new config contents")) return 8;
        // A syntactically valid but semantically invalid primary must not be
        // promoted over a known-good .bak on the next user-initiated Save.
        const auto semantic_path = directory / "semantic-save.json";
        auto semantic_backup = semantic_path;
        semantic_backup += L".bak";
        vietflow::Config known;
        known.macros["golden-backup"] = "keep-this";
        if (!check(known.Save(semantic_path.string()), "seed valid primary")) return 11;
        vietflow::Config changed = known;
        changed.method = 2;
        if (!check(changed.Save(semantic_path.string()), "establish known-good backup")) return 12;
        const std::string valid_backup = bytes(semantic_backup);
        const std::string semantic_bad = R"({"enabled":false,"method":"wrong-type"})";
        {
            std::ofstream out(semantic_path, std::ios::binary | std::ios::trunc);
            out << semantic_bad;
        }
        if (!check(changed.Save(semantic_path.string()), "save with semantic-corrupt primary")) return 13;
        if (!check(bytes(semantic_backup) == valid_backup,
                   "semantically corrupt JSON never replaces a valid backup")) return 14;
        bool found_quarantine = false;
        for (const auto& entry : fs::directory_iterator(directory)) {
            const auto filename = entry.path().filename().string();
            if (filename.find("semantic-save.json.corrupt.") == 0) {
                found_quarantine |= bytes(entry.path()) == semantic_bad;
            }
        }
        if (!check(found_quarantine, "semantic-corrupt primary quarantined verbatim")) return 15;
        vietflow::Config after_semantic_save;
        if (!check(after_semantic_save.Load(semantic_path.string()),
                   "repaired semantic config loads") ||
            !check(after_semantic_save.method == 2,
                   "replacement did not lose the new settings")) return 16;

        // Verify cross-PROCESS writer exclusion (not merely two threads).
        // The parent and children only access this one ephemeral test path.
        const auto simultaneous = directory / "concurrent-settings.json";
        vietflow::Config seed;
        seed.method = 1;
        seed.enabled = true;
        seed.macros["writer-marker"] = "writer-one";
        if (!check(seed.Save(simultaneous.string()), "seed shared writer fixture")) return 17;

        PROCESS_INFORMATION one = {}, two = {};
        if (!check(spawn_writer(simultaneous, 1, one), "spawn writer one")) return 18;
        if (!spawn_writer(simultaneous, 2, two)) {
            TerminateProcess(one.hProcess, 99);
            WaitForSingleObject(one.hProcess, 5000);
            CloseHandle(one.hThread);
            CloseHandle(one.hProcess);
            return 19;
        }
        HANDLE handles[2] = {one.hProcess, two.hProcess};
        const DWORD waited = WaitForMultipleObjects(2, handles, TRUE, 60000);
        if (waited != WAIT_OBJECT_0) {
            TerminateProcess(one.hProcess, 98);
            TerminateProcess(two.hProcess, 98);
            WaitForMultipleObjects(2, handles, TRUE, 5000);
        }
        DWORD code_one = 0, code_two = 0;
        GetExitCodeProcess(one.hProcess, &code_one);
        GetExitCodeProcess(two.hProcess, &code_two);
        CloseHandle(one.hThread);
        CloseHandle(two.hThread);
        CloseHandle(one.hProcess);
        CloseHandle(two.hProcess);
        if (!check(waited == WAIT_OBJECT_0 && code_one == 0 && code_two == 0,
                   "both independent config writers must complete")) return 20;

        auto verify_consistent = [](const fs::path& config_path) {
            vietflow::Config saved;
            if (!saved.Load(config_path.string(), false)) return false;
            const auto it = saved.macros.find("writer-marker");
            if (it == saved.macros.end()) return false;
            return (saved.method == 1 && saved.enabled && it->second == "writer-one") ||
                   (saved.method == 2 && !saved.enabled && it->second == "writer-two");
        };
        auto simultaneous_backup = simultaneous;
        simultaneous_backup += L".bak";
        if (!check(verify_consistent(simultaneous), "final primary is coherent") ||
            !check(verify_consistent(simultaneous_backup), "final backup is coherent")) return 21;
        for (const auto& entry : fs::directory_iterator(directory)) {
            const auto filename = entry.path().filename().string();
            if (filename.find("concurrent-settings.json.tmp.") == 0 ||
                filename.find("concurrent-settings.json.bak.tmp.") == 0) {
                if (!check(false, "a concurrent Save left an incomplete staging file")) return 22;
            }
        }
        std::cout << "PASS: C++ config semantic validation and 2-process atomic save\n";
        return 0;
    } catch (const std::exception& ex) {
        std::cerr << "FAIL: exception: " << ex.what() << "\n";
        return 99;
    }
}
