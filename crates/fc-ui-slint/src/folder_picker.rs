//! Cross-platform native folder picker helpers.

use std::path::PathBuf;
use std::process::Command;

/// Opens a native folder picker and returns selected folder path.
/// Returns `None` when user cancels or no picker backend is available.
pub fn pick_folder() -> Option<PathBuf> {
    match pick_folder_impl() {
        Ok(path) => path,
        Err(err) => {
            tracing::warn!("folder picker failed: {err}");
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn pick_folder_impl() -> anyhow::Result<Option<PathBuf>> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("set pickedFolder to choose folder with prompt \"Select folder\"")
        .arg("-e")
        .arg("POSIX path of pickedFolder")
        .output()?;

    parse_command_output(output.status.success(), &output.stdout, &output.stderr)
}

#[cfg(target_os = "windows")]
fn pick_folder_impl() -> anyhow::Result<Option<PathBuf>> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms;
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog;
$dialog.Description = "Select folder";
$dialog.ShowNewFolderButton = $false;
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dialog.SelectedPath;
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()?;

    parse_command_output(output.status.success(), &output.stdout, &output.stderr)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn pick_folder_impl() -> anyhow::Result<Option<PathBuf>> {
    if let Some(selected) = pick_with_command(
        "zenity",
        &["--file-selection", "--directory", "--title=Select folder"],
    )? {
        return Ok(Some(selected));
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    if let Some(selected) = pick_with_command("kdialog", &["--getexistingdirectory", &home])? {
        return Ok(Some(selected));
    }

    Ok(None)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
fn pick_folder_impl() -> anyhow::Result<Option<PathBuf>> {
    Ok(None)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn pick_with_command(program: &str, args: &[&str]) -> anyhow::Result<Option<PathBuf>> {
    let output = match Command::new(program).args(args).output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    parse_command_output(output.status.success(), &output.stdout, &output.stderr)
}

fn parse_command_output(
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
) -> anyhow::Result<Option<PathBuf>> {
    if success {
        let selected = String::from_utf8_lossy(stdout).trim().to_string();
        if selected.is_empty() {
            return Ok(None);
        }
        return Ok(Some(PathBuf::from(selected)));
    }

    let err_text = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    if err_text.trim().is_empty() {
        return Ok(None);
    }
    if err_text.contains("canceled")
        || err_text.contains("cancelled")
        || err_text.contains("(-128)")
    {
        return Ok(None);
    }

    Err(anyhow::anyhow!(
        "native folder picker returned error: {}",
        String::from_utf8_lossy(stderr).trim()
    ))
}
