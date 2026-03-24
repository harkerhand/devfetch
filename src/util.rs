use std::env;
use std::fs;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const NOT_FOUND: &str = "Not Found";
pub const NA: &str = "N/A";

fn which_cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn command_timeout_ms() -> u64 {
    env::var("RSENV_CMD_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2500)
}

fn run_with_timeout(mut cmd: Command) -> Result<(String, String, bool), String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let timeout = Duration::from_millis(command_timeout_ms());
    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child.wait_with_output().map_err(|e| e.to_string())?;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                return Ok((stdout, stderr, false));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok((String::new(), String::new(), true));
                }
                thread::sleep(Duration::from_millis(25));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

pub fn run_shell(command: &str) -> String {
    #[cfg(target_os = "windows")]
    let cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    };
    #[cfg(not(target_os = "windows"))]
    let cmd = {
        let mut c = Command::new("sh");
        c.arg("-lc").arg(command);
        c
    };
    match run_with_timeout(cmd) {
        Ok((stdout, _stderr, timed_out)) => {
            if timed_out {
                String::new()
            } else {
                stdout.trim().to_string()
            }
        }
        Err(_) => String::new(),
    }
}

pub fn run_shell_unified(command: &str) -> String {
    #[cfg(target_os = "windows")]
    let cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    };
    #[cfg(not(target_os = "windows"))]
    let cmd = {
        let mut c = Command::new("sh");
        c.arg("-lc").arg(command);
        c
    };
    match run_with_timeout(cmd) {
        Ok((stdout, stderr, timed_out)) => {
            if timed_out {
                String::new()
            } else {
                format!("{}{}", stdout, stderr).trim().to_string()
            }
        }
        Err(_) => String::new(),
    }
}

pub fn which(binary: &str) -> Option<String> {
    if let Ok(cache) = which_cache().lock()
        && let Some(cached) = cache.get(binary)
    {
        return cached.clone();
    }

    #[cfg(target_os = "windows")]
    let found = {
        let direct = run_shell(&format!("where {}", shell_escape(binary)));
        if direct.is_empty() {
            String::new()
        } else {
            direct.lines().next().unwrap_or("").to_string()
        }
    };
    #[cfg(not(target_os = "windows"))]
    let found = run_shell(&format!("command -v {}", shell_escape(binary)));
    let resolved = if found.is_empty() {
        None
    } else {
        Some(condense_home(&found))
    };

    if let Ok(mut cache) = which_cache().lock() {
        cache.insert(binary.to_string(), resolved.clone());
    }

    resolved
}

pub fn condense_home(path: &str) -> String {
    if let Ok(home) = env::var("HOME")
        && path.starts_with(&home)
    {
        return path.replacen(&home, "~", 1);
    }
    path.to_string()
}

pub fn find_version(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let mut j = i;
            let mut has_dot = false;
            while j < chars.len() {
                let c = chars[j];
                if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                    if c == '.' {
                        has_dot = true;
                    }
                    j += 1;
                } else {
                    break;
                }
            }
            if has_dot || (j - i) > 1 {
                let v: String = chars[i..j].iter().collect();
                return Some(v.trim_matches('.').to_string());
            }
            i = j;
        }
        i += 1;
    }
    None
}

pub fn to_readable_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 Bytes".to_string();
    }
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut idx = 0usize;
    while size >= 1024.0 && idx < units.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    format!("{size:.2} {}", units[idx])
}

pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

pub fn shell_escape(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/' | '.')) {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn simple_glob_match(pattern: &str, input: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let s: Vec<char> = input.chars().collect();
    let (mut pi, mut si) = (0usize, 0usize);
    let (mut star, mut match_idx) = (None, 0usize);

    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            pi += 1;
            match_idx = si;
        } else if let Some(star_pos) = star {
            pi = star_pos + 1;
            match_idx += 1;
            si = match_idx;
        } else {
            return false;
        }
    }

    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }

    pi == p.len()
}

pub fn run_powershell(command: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        let out = Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .output();
        return match out {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                format!("{}{}", stdout, stderr).trim().to_string()
            }
            Err(_) => String::new(),
        };
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = command;
        String::new()
    }
}

pub fn macos_app_version(bundle_id: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let query = format!("mdfind \"kMDItemCFBundleIdentifier=='{}'\" | head -1", bundle_id);
        let app_path = run_shell(&query);
        if app_path.is_empty() {
            return None;
        }
        let plist = format!("{}/Contents/Info.plist", app_path.trim());
        let short = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", "Print:CFBundleShortVersionString", &plist])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if !short.is_empty() {
            return Some(short);
        }
        let build = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", "Print:CFBundleVersion", &plist])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if build.is_empty() {
            None
        } else {
            Some(build)
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bundle_id;
        None
    }
}

pub fn windows_file_version(path: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let cmd = format!(
            "(Get-Item '{}').VersionInfo.ProductVersion",
            path.replace('\'', "''")
        );
        let out = run_powershell(&cmd);
        if out.is_empty() {
            None
        } else {
            Some(out.lines().next().unwrap_or("").trim().to_string())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        None
    }
}
