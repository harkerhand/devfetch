use std::collections::BTreeMap;
use std::env;
use std::sync::mpsc;
use std::thread;

use crate::model::{Node, RunOptions};
use crate::presets::PresetSpec;
use crate::util::{self, NA, NOT_FOUND};

fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

fn version_path(name: &str, version: Option<String>, path: Option<String>) -> Node {
    match version {
        Some(v) if !v.is_empty() => {
            if let Some(p) = path {
                let mut m = BTreeMap::new();
                m.insert("version".to_string(), Node::Str(v));
                m.insert("path".to_string(), Node::Str(p));
                Node::Obj(m)
            } else {
                Node::Str(v)
            }
        }
        _ => {
            let _ = name;
            Node::Str(NOT_FOUND.to_string())
        }
    }
}

fn run_probe(name: &str, binary: &str, cmd: &str) -> Node {
    let path = util::which(binary);
    if path.is_none() {
        return Node::Str(NOT_FOUND.to_string());
    }
    let out = util::run_shell_unified(cmd);
    let version = util::find_version(&out);
    version_path(name, version, path)
}

fn version_only_from_cmd(cmd: &str) -> Node {
    let out = util::run_shell_unified(cmd);
    match util::find_version(&out) {
        Some(v) if !v.is_empty() => Node::Str(v),
        _ => Node::Str(NOT_FOUND.to_string()),
    }
}

fn version_only(v: Option<String>) -> Node {
    match v {
        Some(s) if !s.is_empty() => Node::Str(s),
        _ => Node::Str(NOT_FOUND.to_string()),
    }
}

fn mac_app_version(bundle_id: &str) -> Node {
    if !is_macos() {
        return Node::Str(NA.to_string());
    }
    version_only(util::macos_app_version(bundle_id))
}

fn windows_file_version(paths: &[&str]) -> Node {
    if !is_windows() {
        return Node::Str(NA.to_string());
    }
    for path in paths {
        if let Some(v) = util::windows_file_version(path)
            && !v.is_empty()
        {
            return Node::Str(v);
        }
    }
    Node::Str(NOT_FOUND.to_string())
}

fn shell_name_from_env() -> Option<String> {
    env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(|x| x.to_string()))
}

fn collect_system_item(item: &str) -> Node {
    match item {
        "OS" => {
            if is_linux()
                && let Some(content) = util::read_file("/etc/os-release")
            {
                let name = content
                    .lines()
                    .find(|l| l.starts_with("NAME="))
                    .map(|l| l.trim_start_matches("NAME=").trim_matches('"'))
                    .unwrap_or("Linux");
                let version = content
                    .lines()
                    .find(|l| l.starts_with("VERSION="))
                    .map(|l| l.trim_start_matches("VERSION=").trim_matches('"'))
                    .unwrap_or("");
                return Node::Str(format!("{} {}", name, version).trim().to_string());
            }
            if is_macos() {
                let v = util::run_shell("sw_vers -productVersion");
                if !v.is_empty() {
                    return Node::Str(format!("macOS {v}"));
                }
            }
            if is_windows() {
                let caption =
                    util::run_powershell("(Get-CimInstance Win32_OperatingSystem).Caption");
                let version =
                    util::run_powershell("(Get-CimInstance Win32_OperatingSystem).Version");
                let caption = caption.lines().next().unwrap_or("").trim();
                let version = version.lines().next().unwrap_or("").trim();
                if !caption.is_empty() && !version.is_empty() {
                    return Node::Str(format!("{caption} {version}"));
                }
            }
            let uname = util::run_shell("uname -sr");
            if uname.is_empty() {
                Node::Str(std::env::consts::OS.to_string())
            } else {
                Node::Str(uname)
            }
        }
        "CPU" => {
            let arch = std::env::consts::ARCH;
            let cores = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1usize);
            let model = if is_linux() {
                util::read_file("/proc/cpuinfo")
                    .and_then(|c| {
                        c.lines()
                            .find(|l| l.starts_with("model name"))
                            .and_then(|l| l.split_once(':').map(|(_, r)| r.trim().to_string()))
                    })
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                util::run_shell("sysctl -n machdep.cpu.brand_string")
            };
            Node::Str(format!("({cores}) {arch} {model}"))
        }
        "Memory" => {
            if is_linux() {
                let meminfo = util::read_file("/proc/meminfo").unwrap_or_default();
                let total_kb = meminfo
                    .lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|n| n.parse::<u64>().ok())
                    .unwrap_or(0);
                let avail_kb = meminfo
                    .lines()
                    .find(|l| l.starts_with("MemAvailable:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|n| n.parse::<u64>().ok())
                    .unwrap_or(0);
                Node::Str(format!(
                    "{} / {}",
                    util::to_readable_bytes(avail_kb * 1024),
                    util::to_readable_bytes(total_kb * 1024)
                ))
            } else if is_macos() {
                let total = util::run_shell("sysctl -n hw.memsize")
                    .parse::<u64>()
                    .unwrap_or(0);
                let free_pages =
                    util::run_shell("vm_stat | awk '/Pages free/ {print $3}' | tr -d '.'")
                        .parse::<u64>()
                        .unwrap_or(0);
                let inactive_pages =
                    util::run_shell("vm_stat | awk '/Pages inactive/ {print $3}' | tr -d '.'")
                        .parse::<u64>()
                        .unwrap_or(0);
                let page_size = util::run_shell("pagesize").parse::<u64>().unwrap_or(4096);
                let free = (free_pages + inactive_pages) * page_size;
                if total > 0 {
                    Node::Str(format!(
                        "{} / {}",
                        util::to_readable_bytes(free),
                        util::to_readable_bytes(total)
                    ))
                } else {
                    Node::Str(NA.to_string())
                }
            } else if is_windows() {
                let total_kb = util::run_powershell(
                    "(Get-CimInstance Win32_OperatingSystem).TotalVisibleMemorySize",
                )
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .parse::<u64>()
                .unwrap_or(0);
                let free_kb = util::run_powershell(
                    "(Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory",
                )
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .parse::<u64>()
                .unwrap_or(0);
                if total_kb > 0 {
                    Node::Str(format!(
                        "{} / {}",
                        util::to_readable_bytes(free_kb * 1024),
                        util::to_readable_bytes(total_kb * 1024)
                    ))
                } else {
                    Node::Str(NA.to_string())
                }
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Container" => {
            if is_linux() {
                let has_docker_env = util::file_exists("/.dockerenv");
                let cgroup = util::read_file("/proc/self/cgroup").unwrap_or_default();
                let in_container = has_docker_env
                    || cgroup.contains("docker")
                    || cgroup.contains("containerd")
                    || cgroup.contains("kubepods");
                Node::Str(if in_container { "Yes" } else { "N/A" }.to_string())
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Shell" => {
            if is_linux() || is_macos() {
                let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
                let cmd = format!("{} --version 2>&1", util::shell_escape(&shell));
                let out = util::run_shell_unified(&cmd);
                let version = util::find_version(&out).or_else(|| Some("Unknown".to_string()));
                let path =
                    util::which(&shell).or_else(|| util::which(shell_name_from_env()?.as_str()));
                version_path("Shell", version, path)
            } else {
                Node::Str(NA.to_string())
            }
        }
        _ => Node::Str(NOT_FOUND.to_string()),
    }
}

fn collect_sdk_item(item: &str) -> Node {
    match item {
        "iOS SDK" => {
            if !is_macos() {
                return Node::Str(NA.to_string());
            }
            let out = util::run_shell("xcodebuild -showsdks");
            let mut vals = Vec::new();
            for line in out.lines() {
                if let Some(v) = util::find_version(line) {
                    if line.contains("iOS")
                        || line.contains("macOS")
                        || line.contains("tvOS")
                        || line.contains("watchOS")
                    {
                        vals.push(line.trim().to_string());
                    }
                    if vals.len() > 12 {
                        break;
                    }
                    let _ = v;
                }
            }
            if vals.is_empty() {
                Node::Str(NOT_FOUND.to_string())
            } else {
                let mut obj = BTreeMap::new();
                obj.insert(
                    "Platforms".to_string(),
                    Node::Arr(vals.into_iter().map(Node::Str).collect()),
                );
                Node::Obj(obj)
            }
        }
        "Android SDK" => {
            let mut out = util::run_shell("sdkmanager --list");
            if out.is_empty()
                && let Ok(home) = env::var("ANDROID_HOME")
            {
                out = util::run_shell(&format!(
                    "{home}/cmdline-tools/latest/bin/sdkmanager --list"
                ));
            }
            if out.is_empty()
                && is_windows()
                && let Ok(home) = env::var("USERPROFILE")
            {
                out = util::run_shell(&format!(
                    "\"{}\\AppData\\Local\\Android\\Sdk\\cmdline-tools\\latest\\bin\\sdkmanager.bat\" --list",
                    home
                ));
            }
            if out.is_empty() && is_macos() {
                out = util::run_shell(
                    "~/Library/Android/sdk/cmdline-tools/latest/bin/sdkmanager --list",
                );
            }
            if out.is_empty() {
                return Node::Str(NOT_FOUND.to_string());
            }
            let installed = out.split("Available").next().unwrap_or(&out);
            let mut api = Vec::new();
            let mut build_tools = Vec::new();
            let mut images = Vec::new();
            for line in installed.lines() {
                let l = line.trim();
                if let Some(idx) = l.find("platforms;android-") {
                    let chunk = &l[idx + "platforms;android-".len()..];
                    let level: String = chunk.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if !level.is_empty() {
                        api.push(level);
                    }
                }
                if let Some(idx) = l.find("build-tools;") {
                    let chunk = &l[idx + "build-tools;".len()..];
                    let bt: String = chunk
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !bt.is_empty() {
                        build_tools.push(bt);
                    }
                }
                if let Some(idx) = l.find("system-images;") {
                    images.push(l[idx + "system-images;".len()..].to_string());
                }
            }
            let mut obj = BTreeMap::new();
            obj.insert(
                "API Levels".to_string(),
                Node::Arr(api.into_iter().map(Node::Str).collect()),
            );
            obj.insert(
                "Build Tools".to_string(),
                Node::Arr(build_tools.into_iter().map(Node::Str).collect()),
            );
            obj.insert(
                "System Images".to_string(),
                Node::Arr(images.into_iter().map(Node::Str).collect()),
            );
            Node::Obj(obj)
        }
        "Windows SDK" => {
            if is_windows() {
                let out = util::run_shell(
                    "reg query HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock",
                );
                if out.is_empty() {
                    Node::Str(NOT_FOUND.to_string())
                } else {
                    let mut obj = BTreeMap::new();
                    for line in out.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3
                            && parts[0]
                                != "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock"
                        {
                            let key = parts[0].to_string();
                            let value = parts[2..].join(" ");
                            obj.insert(key, Node::Str(value));
                        }
                    }
                    if obj.is_empty() {
                        Node::Str(NOT_FOUND.to_string())
                    } else {
                        Node::Obj(obj)
                    }
                }
            } else {
                Node::Str(NA.to_string())
            }
        }
        _ => Node::Str(NOT_FOUND.to_string()),
    }
}

fn collect_generic_item(item: &str) -> Node {
    match item {
        "Node" => run_probe("Node", "node", "node -v"),
        "Yarn" => run_probe("Yarn", "yarn", "yarn -v"),
        "npm" => run_probe("npm", "npm", "npm -v"),
        "pnpm" => run_probe("pnpm", "pnpm", "pnpm -v"),
        "bun" => run_probe("bun", "bun", "bun -v"),
        "Deno" => run_probe("Deno", "deno", "deno --version"),
        "Watchman" => run_probe("Watchman", "watchman", "watchman -v"),

        "Apt" => {
            if is_linux() {
                run_probe("Apt", "apt", "apt --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Cargo" => run_probe("Cargo", "cargo", "cargo --version"),
        "CocoaPods" => {
            if is_macos() {
                run_probe("CocoaPods", "pod", "pod --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Composer" => run_probe("Composer", "composer", "composer --version"),
        "Gradle" => run_probe("Gradle", "gradle", "gradle --version"),
        "Homebrew" => {
            if is_linux() || is_macos() {
                run_probe("Homebrew", "brew", "brew --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Maven" => run_probe("Maven", "mvn", "mvn --version"),
        "pip2" => run_probe("pip2", "pip2", "pip2 --version"),
        "pip3" => run_probe("pip3", "pip3", "pip3 --version"),
        "RubyGems" => run_probe("RubyGems", "gem", "gem --version"),
        "Yum" => {
            if is_linux() {
                run_probe("Yum", "yum", "yum --version")
            } else {
                Node::Str(NA.to_string())
            }
        }

        "7z" => {
            if util::which("7z").is_some() {
                run_probe("7z", "7z", "7z i")
            } else {
                run_probe("7z", "7zz", "7zz i")
            }
        }
        "Bazel" => run_probe("Bazel", "bazel", "bazel --version"),
        "CMake" => run_probe("CMake", "cmake", "cmake --version"),
        "Make" => {
            if is_linux() || is_macos() {
                run_probe("Make", "make", "make --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "GCC" => {
            if is_linux() || is_macos() {
                run_probe("GCC", "gcc", "gcc -v")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Git" => run_probe("Git", "git", "git --version"),
        "Git LFS" => run_probe("Git LFS", "git-lfs", "git lfs version"),
        "Clang" => run_probe("Clang", "clang", "clang --version"),
        "Ninja" => run_probe("Ninja", "ninja", "ninja --version"),
        "Mercurial" => {
            if is_linux() || is_macos() {
                run_probe("Mercurial", "hg", "hg --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Subversion" => {
            if is_linux() || is_macos() {
                run_probe("Subversion", "svn", "svn --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "FFmpeg" => run_probe("FFmpeg", "ffmpeg", "ffmpeg -version"),
        "Curl" => run_probe("Curl", "curl", "curl --version"),
        "OpenSSL" => run_probe("OpenSSL", "openssl", "openssl version"),
        "ccache" => run_probe("ccache", "ccache", "ccache -V"),
        "Calibre" => run_probe("Calibre", "ebook-convert", "ebook-convert --version"),
        "Clash Meta" => run_probe("Clash Meta", "mihomo", "mihomo -v"),

        "Apache" => {
            if is_linux() || is_macos() {
                run_probe("Apache", "apachectl", "apachectl -v")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Nginx" => {
            if is_linux() || is_macos() {
                run_probe("Nginx", "nginx", "nginx -v")
            } else {
                Node::Str(NA.to_string())
            }
        }

        "Docker" => run_probe("Docker", "docker", "docker --version"),
        "Docker Compose" => run_probe(
            "Docker Compose",
            "docker-compose",
            "docker-compose --version",
        ),
        "Parallels" => run_probe("Parallels", "prlctl", "prlctl --version"),
        "VirtualBox" => run_probe("VirtualBox", "vboxmanage", "vboxmanage --version"),
        "VMware Fusion" => {
            if is_macos() {
                run_probe("VMware Fusion", "vmrun", "vmrun -v")
            } else {
                Node::Str(NA.to_string())
            }
        }

        "Android Studio" => {
            if is_linux() {
                let out = util::run_shell("cat /opt/android-studio/build.txt");
                if out.is_empty() {
                    Node::Str(NOT_FOUND.to_string())
                } else {
                    Node::Str(out)
                }
            } else if is_macos() {
                mac_app_version("com.google.android.studio")
            } else if is_windows() {
                windows_file_version(&[
                    "C:\\Program Files\\Android\\Android Studio\\bin\\studio64.exe",
                    "C:\\Program Files\\Android\\Android Studio\\bin\\studio.exe",
                ])
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Atom" => {
            if is_macos() {
                mac_app_version("com.github.atom")
            } else {
                run_probe("Atom", "atom", "atom --version")
            }
        }
        "Emacs" => {
            if is_linux() || is_macos() {
                run_probe("Emacs", "emacs", "emacs --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "IntelliJ" => {
            if is_macos() {
                mac_app_version("com.jetbrains.intellij")
            } else {
                run_probe("IntelliJ", "idea", "idea --version")
            }
        }
        "NVim" => {
            if is_linux() || is_macos() {
                run_probe("NVim", "nvim", "nvim --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Nano" => {
            if is_linux() {
                run_probe("Nano", "nano", "nano --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "PhpStorm" => {
            if is_macos() {
                mac_app_version("com.jetbrains.PhpStorm")
            } else {
                run_probe("PhpStorm", "phpstorm", "phpstorm --version")
            }
        }
        "Sublime Text" => {
            if is_macos() {
                mac_app_version("com.sublimetext.3")
            } else {
                run_probe("Sublime Text", "subl", "subl --version")
            }
        }
        "VSCode" => run_probe("VSCode", "code", "code --version"),
        "Cursor" => run_probe("Cursor", "cursor", "cursor --version"),
        "Claude Code" => run_probe("Claude Code", "claude", "claude --version"),
        "Codex" => run_probe("Codex", "codex", "codex --version"),
        "opencode" => run_probe("opencode", "opencode", "opencode --version"),
        "Visual Studio" => {
            if is_windows() {
                let out = util::run_shell(
                    "\"%ProgramFiles(x86)%\\Microsoft Visual Studio\\Installer\\vswhere.exe\" -format json -prerelease",
                );
                if out.is_empty() {
                    Node::Str(NOT_FOUND.to_string())
                } else {
                    Node::Str(out)
                }
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Vim" => {
            if is_linux() || is_macos() {
                run_probe("Vim", "vim", "vim --version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "WebStorm" => {
            if is_macos() {
                mac_app_version("com.jetbrains.WebStorm")
            } else {
                run_probe("WebStorm", "webstorm", "webstorm --version")
            }
        }
        "Xcode" => {
            if is_macos() {
                version_only_from_cmd("xcodebuild -version")
            } else {
                Node::Str(NA.to_string())
            }
        }

        "Bash" => run_probe("Bash", "bash", "bash --version"),
        "Go" => run_probe("Go", "go", "go version"),
        "Elixir" => run_probe("Elixir", "elixir", "elixir --version"),
        "Erlang" => run_probe("Erlang", "erl", "erl -eval \"halt().\" -noshell"),
        "Java" => run_probe("Java", "javac", "javac -version"),
        "Perl" => run_probe("Perl", "perl", "perl -v"),
        "PHP" => run_probe("PHP", "php", "php -v"),
        "Protoc" => run_probe("Protoc", "protoc", "protoc --version"),
        "Python" => run_probe("Python", "python", "python -V"),
        "Python3" => run_probe("Python3", "python3", "python3 -V"),
        "R" => run_probe("R", "R", "R --version"),
        "Ruby" => run_probe("Ruby", "ruby", "ruby -v"),
        "Rust" => run_probe("Rust", "rustc", "rustc --version"),
        "Scala" => {
            if is_linux() || is_macos() {
                run_probe("Scala", "scalac", "scalac -version")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Zig" => run_probe("Zig", "zig", "zig version"),

        "MongoDB" => run_probe("MongoDB", "mongo", "mongo --version"),
        "MySQL" => run_probe("MySQL", "mysql", "mysql --version"),
        "PostgreSQL" => run_probe("PostgreSQL", "postgres", "postgres --version"),
        "SQLite" => run_probe("SQLite", "sqlite3", "sqlite3 --version"),

        "Brave Browser" => {
            if is_linux() {
                version_only_from_cmd("brave --version || brave-browser --version")
            } else if is_macos() {
                mac_app_version("com.brave.Browser")
            } else if is_windows() {
                windows_file_version(&[
                    "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
                    "C:\\Program Files (x86)\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
                ])
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Chrome" => {
            if is_linux() {
                version_only_from_cmd("google-chrome --version")
            } else if is_macos() {
                mac_app_version("com.google.Chrome")
            } else if is_windows() {
                windows_file_version(&[
                    "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                    "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
                ])
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Chrome Canary" => {
            if is_macos() {
                mac_app_version("com.google.Chrome.canary")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Chromium" => {
            if is_linux() {
                version_only_from_cmd("chromium --version")
            } else if is_windows() {
                windows_file_version(&["C:\\Program Files\\Chromium\\Application\\chrome.exe"])
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Edge" => {
            if is_macos() {
                mac_app_version("com.microsoft.edgemac")
            } else if is_linux() {
                version_only_from_cmd("microsoft-edge --version")
            } else if is_windows() {
                windows_file_version(&[
                    "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
                    "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
                ])
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Firefox" => {
            if is_macos() {
                mac_app_version("org.mozilla.firefox")
            } else if is_windows() {
                windows_file_version(&[
                    "C:\\Program Files\\Mozilla Firefox\\firefox.exe",
                    "C:\\Program Files (x86)\\Mozilla Firefox\\firefox.exe",
                ])
            } else {
                version_only_from_cmd("firefox --version")
            }
        }
        "Firefox Developer Edition" => {
            if is_macos() {
                mac_app_version("org.mozilla.firefoxdeveloperedition")
            } else if is_windows() {
                windows_file_version(&["C:\\Program Files\\Firefox Developer Edition\\firefox.exe"])
            } else {
                version_only_from_cmd("firefox --version")
            }
        }
        "Firefox Nightly" => {
            if is_macos() {
                mac_app_version("org.mozilla.nightly")
            } else if is_windows() {
                windows_file_version(&["C:\\Program Files\\Firefox Nightly\\firefox.exe"])
            } else {
                version_only_from_cmd("firefox-trunk --version || firefox --version")
            }
        }
        "Internet Explorer" => {
            if is_windows() {
                windows_file_version(&["C:\\Program Files\\Internet Explorer\\iexplore.exe"])
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Safari" => {
            if is_macos() {
                mac_app_version("com.apple.Safari")
            } else {
                Node::Str(NA.to_string())
            }
        }
        "Safari Technology Preview" => {
            if is_macos() {
                mac_app_version("com.apple.SafariTechnologyPreview")
            } else {
                Node::Str(NA.to_string())
            }
        }

        _ => Node::Str(NOT_FOUND.to_string()),
    }
}

fn collect_item(category: &str, item: &str) -> Node {
    match category {
        "System" => collect_system_item(item),
        "SDKs" => collect_sdk_item(item),
        _ => collect_generic_item(item),
    }
}

pub fn collect_report(spec: &PresetSpec, _options: &RunOptions) -> Node {
    let mut report = BTreeMap::new();
    let ordered: Vec<(String, Option<Vec<String>>)> = spec
        .iter()
        .map(|(category, maybe_items)| (category.clone(), maybe_items.clone()))
        .collect();

    let sections = thread::scope(|scope| {
        let (tx, rx) = mpsc::channel::<(String, String, Node)>();

        for (category, maybe_items) in &ordered {
            if let Some(items) = maybe_items {
                for item in items {
                    let tx = tx.clone();
                    let cat = category.clone();
                    let it = item.clone();
                    scope.spawn(move || {
                        let node = collect_item(&cat, &it);
                        let _ = tx.send((cat, it, node));
                    });
                }
            }
        }
        drop(tx);

        let mut out: BTreeMap<String, BTreeMap<String, Node>> = BTreeMap::new();
        for (cat, item, node) in rx {
            out.entry(cat).or_default().insert(item, node);
        }
        out
    });

    for (category, maybe_items) in ordered {
        let mut section = sections.get(&category).cloned().unwrap_or_default();
        if let Some(items) = maybe_items {
            // Preserve original key order determinism via insertion into BTreeMap is sorted.
            for item in items {
                section
                    .entry(item)
                    .or_insert_with(|| Node::Str(NOT_FOUND.to_string()));
            }
        }
        report.insert(category, Node::Obj(section));
    }

    Node::Obj(report)
}

pub fn collect_helper(name: &str, _options: &RunOptions) -> Option<Node> {
    let mut raw = name.trim().to_string();
    if let Some(stripped) = raw.strip_prefix("get") {
        raw = stripped.to_string();
    }
    if let Some(stripped) = raw.strip_suffix("Info") {
        raw = stripped.to_string();
    }

    let all = crate::presets::defaults();
    for (category, items) in all {
        if let Some(items) = items {
            for item in items {
                let norm_item = item.to_ascii_lowercase().replace(' ', "");
                let norm_raw = raw.to_ascii_lowercase().replace(' ', "");
                if norm_item == norm_raw {
                    return Some(collect_item(&category, &item));
                }
            }
        }
    }

    None
}
