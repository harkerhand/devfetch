use std::collections::BTreeMap;

pub type PresetSpec = BTreeMap<String, Option<Vec<String>>>;

fn v(items: &[&str]) -> Option<Vec<String>> {
    Some(items.iter().map(|s| s.to_string()).collect())
}

pub fn defaults() -> PresetSpec {
    let mut m = BTreeMap::new();
    m.insert(
        "System".to_string(),
        v(&["OS", "CPU", "Memory", "Container", "Shell"]),
    );
    m.insert(
        "Binaries".to_string(),
        v(&["Node", "Yarn", "npm", "pnpm", "bun", "Deno", "Watchman"]),
    );
    m.insert(
        "Managers".to_string(),
        v(&[
            "Apt",
            "Cargo",
            "CocoaPods",
            "Composer",
            "Gradle",
            "Homebrew",
            "Maven",
            "pip2",
            "pip3",
            "RubyGems",
            "Yum",
        ]),
    );
    m.insert(
        "Utilities".to_string(),
        v(&[
            "7z",
            "Bazel",
            "CMake",
            "Make",
            "GCC",
            "Git",
            "Git LFS",
            "Clang",
            "Ninja",
            "Mercurial",
            "Subversion",
            "FFmpeg",
            "Curl",
            "OpenSSL",
            "ccache",
            "Calibre",
            "Clash Meta",
        ]),
    );
    m.insert("Servers".to_string(), v(&["Apache", "Nginx"]));
    m.insert(
        "Virtualization".to_string(),
        v(&[
            "Docker",
            "Docker Compose",
            "Parallels",
            "VirtualBox",
            "VMware Fusion",
        ]),
    );
    m.insert(
        "SDKs".to_string(),
        v(&["iOS SDK", "Android SDK", "Windows SDK"]),
    );
    m.insert(
        "IDEs".to_string(),
        v(&[
            "Android Studio",
            "Atom",
            "Emacs",
            "IntelliJ",
            "NVim",
            "Nano",
            "PhpStorm",
            "Sublime Text",
            "VSCode",
            "Cursor",
            "Claude Code",
            "Codex",
            "opencode",
            "Visual Studio",
            "Vim",
            "WebStorm",
            "Xcode",
        ]),
    );
    m.insert(
        "Languages".to_string(),
        v(&[
            "Bash", "Go", "Elixir", "Erlang", "Java", "Perl", "PHP", "Protoc", "Python",
            "Python3", "R", "Ruby", "Rust", "Scala", "Zig",
        ]),
    );
    m.insert(
        "Databases".to_string(),
        v(&["MongoDB", "MySQL", "PostgreSQL", "SQLite"]),
    );
    m.insert(
        "Browsers".to_string(),
        v(&[
            "Brave Browser",
            "Chrome",
            "Chrome Canary",
            "Chromium",
            "Edge",
            "Firefox",
            "Firefox Developer Edition",
            "Firefox Nightly",
            "Internet Explorer",
            "Safari",
            "Safari Technology Preview",
        ]),
    );
    m
}
