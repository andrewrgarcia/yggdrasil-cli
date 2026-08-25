//! Copying a codex to the system clipboard.
//!
//! Two strategies, in order:
//!
//! 1. A native clipboard tool (`wl-copy`, `xclip`, …). Reliable, unlimited
//!    size, but needs the binary present and a local display.
//! 2. OSC 52 — an escape sequence that asks the *terminal* to set the
//!    clipboard. Needs no binary and works over SSH, but many terminals cap
//!    the payload (commonly ~74k) and some disable it outright.
//!
//! Native first, because a codex routinely exceeds what OSC 52 will carry.

use std::io::Write;
use std::process::{Command, Stdio};

/// How the text reached the clipboard — reported so the user knows whether
/// to trust it for a large codex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Name of the tool that took it on stdin.
    Tool(&'static str),
    /// Terminal escape sequence. Size-capped, and silently ignored by many
    /// terminals (GNOME Terminal among them) with no way to detect it — so
    /// this is reported as "attempted", never as "copied".
    Osc52,
}

/// The platform-appropriate way to get a real clipboard tool.
pub fn install_hint() -> &'static str {
    if cfg!(target_os = "macos") {
        "pbcopy should be built in — check your PATH"
    } else if cfg!(target_os = "windows") {
        "clip.exe should be built in — check your PATH"
    } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
        "install wl-clipboard (apt install wl-clipboard)"
    } else {
        "install xclip (apt install xclip) or wl-clipboard"
    }
}

/// How a tool wants its stdin encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Utf8,
    /// Windows `clip` decodes stdin using the system ANSI codepage, which
    /// mangles every emoji and box-drawing character in a codex. UTF-16LE
    /// with a BOM is the one form it reads unambiguously.
    Utf16Le,
}

/// Candidate clipboard tools, best first.
///
/// Wayland before X11 so a Wayland session doesn't get routed through
/// XWayland. `clip.exe` covers both WSL and native Windows; `clip` is the
/// bare name in case PATHEXT resolution is in play.
const TOOLS: &[(&str, &[&str], Encoding)] = &[
    ("wl-copy", &[], Encoding::Utf8),
    ("xclip", &["-selection", "clipboard"], Encoding::Utf8),
    ("xsel", &["--clipboard", "--input"], Encoding::Utf8),
    ("pbcopy", &[], Encoding::Utf8),
    ("clip.exe", &[], Encoding::Utf16Le),
    ("clip", &[], Encoding::Utf16Le),
    ("termux-clipboard-set", &[], Encoding::Utf8),
];

/// Terminals commonly refuse OSC 52 payloads above roughly this size.
const OSC52_LIMIT: usize = 74_000;

/// Copy `text` to the system clipboard.
pub fn copy(text: &str) -> Result<Route, String> {
    for (tool, args, encoding) in TOOLS {
        match try_tool(tool, args, text, *encoding) {
            Ok(true) => return Ok(Route::Tool(tool)),
            // Not installed, or it failed — keep looking.
            Ok(false) | Err(_) => continue,
        }
    }

    copy_osc52(text).map(|_| Route::Osc52)
}

/// `Ok(false)` means "not installed"; `Err` means it ran and failed.
fn try_tool(
    tool: &str,
    args: &[&str],
    text: &str,
    encoding: Encoding,
) -> Result<bool, String> {
    let spawned = Command::new(tool)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match spawned {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| format!("{tool}: no stdin"))?;
        let payload = match encoding {
            Encoding::Utf8 => text.as_bytes().to_vec(),
            Encoding::Utf16Le => utf16le_with_bom(text),
        };
        stdin
            .write_all(&payload)
            .map_err(|e| format!("{tool}: {e}"))?;
    } // drop stdin so the tool sees EOF

    // wl-copy forks a daemon to own the selection; it exits promptly.
    let status = child.wait().map_err(|e| format!("{tool}: {e}"))?;

    if status.success() {
        Ok(true)
    } else {
        Err(format!("{tool} exited with {status}"))
    }
}

/// Encode as UTF-16LE, BOM first.
fn utf16le_with_bom(text: &str) -> Vec<u8> {
    let mut out = vec![0xff, 0xfe]; // BOM
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

/// Ask the terminal itself to set the clipboard.
///
/// On Unix this goes to `/dev/tty` rather than stdout, because the picker's
/// stdout may be redirected and the sequence has to reach the terminal to
/// mean anything. Windows has no `/dev/tty`, so stdout is the only option
/// there — and in practice `clip` is found first, so this rarely runs.
fn copy_osc52(text: &str) -> Result<(), String> {
    if text.len() > OSC52_LIMIT {
        return Err(format!(
            "no clipboard tool found, and this codex ({} KB) exceeds what the \
             terminal-escape fallback can carry — {}",
            text.len() / 1024,
            install_hint()
        ));
    }

    let payload = base64(text.as_bytes());
    let seq = format!("\x1b]52;c;{payload}\x07");

    #[cfg(unix)]
    {
        let mut tty = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/tty")
            .map_err(|e| format!("no clipboard tool found, and /dev/tty: {e}"))?;

        tty.write_all(seq.as_bytes())
            .map_err(|e| format!("/dev/tty: {e}"))?;
        tty.flush().map_err(|e| format!("/dev/tty: {e}"))?;
    }

    #[cfg(not(unix))]
    {
        let mut stdout = std::io::stdout();
        stdout
            .write_all(seq.as_bytes())
            .map_err(|e| format!("stdout: {e}"))?;
        stdout.flush().map_err(|e| format!("stdout: {e}"))?;
    }

    Ok(())
}

/// Standard base64, padded. Small enough to not be worth a dependency.
fn base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;

        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_handles_high_bytes_and_utf8() {
        // A codex is full of these — 📄, ✨, box-drawing characters.
        assert_eq!(base64("✨".as_bytes()), "4pyo");
        assert_eq!(base64(&[0xff, 0xfe, 0xfd]), "//79");
    }

    #[test]
    fn base64_length_is_always_a_multiple_of_four() {
        for n in 0..64 {
            let input = vec![b'x'; n];
            assert_eq!(base64(&input).len() % 4, 0, "n = {n}");
        }
    }

    #[test]
    fn oversized_payload_is_refused_rather_than_truncated() {
        // Silently copying half a codex would be worse than not copying.
        let huge = "x".repeat(OSC52_LIMIT + 1);
        let err = copy_osc52(&huge).unwrap_err();
        assert!(err.contains("exceeds"), "got: {err}");
    }

    #[test]
    fn missing_tool_reports_absence_not_failure() {
        let got = try_tool(
            "definitely-not-a-real-binary-xyz",
            &[],
            "hi",
            Encoding::Utf8,
        );
        assert_eq!(got, Ok(false));
    }

    #[test]
    fn utf16_encoding_leads_with_a_bom() {
        let got = utf16le_with_bom("hi");
        assert_eq!(got, vec![0xff, 0xfe, b'h', 0x00, b'i', 0x00]);
    }

    #[test]
    fn utf16_survives_the_emoji_a_codex_is_full_of() {
        // 📄 is outside the BMP, so it must come out as a surrogate pair —
        // this is exactly what Windows `clip` mangles when fed UTF-8.
        let got = utf16le_with_bom("📄");
        assert_eq!(got, vec![0xff, 0xfe, 0x3d, 0xd8, 0xc4, 0xdc]);
        // BOM + two UTF-16 code units.
        assert_eq!(got.len(), 6);
    }

    #[test]
    fn every_windows_tool_asks_for_utf16() {
        for (tool, _, encoding) in TOOLS {
            if tool.starts_with("clip") {
                assert_eq!(*encoding, Encoding::Utf16Le, "{tool}");
            }
        }
    }
}