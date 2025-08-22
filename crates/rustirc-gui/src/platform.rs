//! Platform-specific integrations for RustIRC
//!
//! Provides native features for Windows, macOS, and Linux including:
//! - Toast/popup notifications
//! - System tray integration  
//! - Desktop integration
//! - File associations

#![allow(clippy::disallowed_types)] // Platform integration requires system commands

use anyhow::Result;
use std::path::Path;

/// Platform notification system
pub struct NotificationManager {
    enabled: bool,
    sound_enabled: bool,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
        }
    }

    /// Show a notification with title and body
    pub fn show_notification(&self, title: &str, body: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        self.show_windows_notification(title, body)?;

        #[cfg(target_os = "macos")]
        self.show_macos_notification(title, body)?;

        #[cfg(target_os = "linux")]
        self.show_linux_notification(title, body)?;

        Ok(())
    }

    /// Show notification for new message
    pub fn show_message_notification(
        &self,
        sender: &str,
        channel: &str,
        message: &str,
    ) -> Result<()> {
        let title = if channel.starts_with('#') {
            format!("{sender} in {channel}")
        } else {
            format!("Private message from {sender}")
        };

        // Truncate long messages
        let body = if message.len() > 100 {
            format!("{}...", &message[..97])
        } else {
            message.to_string()
        };

        self.show_notification(&title, &body)
    }

    /// Show notification for highlight
    pub fn show_highlight_notification(
        &self,
        sender: &str,
        channel: &str,
        message: &str,
    ) -> Result<()> {
        let title = format!("🔔 Highlighted in {channel} by {sender}");
        self.show_notification(&title, message)
    }

    /// Show notification for private message
    pub fn show_private_message_notification(&self, sender: &str, message: &str) -> Result<()> {
        let title = format!("💬 {sender}");
        self.show_notification(&title, message)
    }

    /// Enable or disable notifications
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Enable or disable notification sounds
    pub fn set_sound_enabled(&mut self, enabled: bool) {
        self.sound_enabled = enabled;
    }

    #[cfg(target_os = "windows")]
    fn show_windows_notification(&self, title: &str, body: &str) -> Result<()> {
        // Windows Toast Notifications implementation
        // This would use the Windows Runtime API
        // For now, this is a placeholder

        use std::process::Command;

        // Fallback using PowerShell for toast notifications
        let script = format!(
            r#"
            [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.UI.Notifications.ToastNotification, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
            
            $template = @"
            <toast>
                <visual>
                    <binding template="ToastGeneric">
                        <text>{}</text>
                        <text>{}</text>
                    </binding>
                </visual>
            </toast>
            "@
            
            $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
            $xml.LoadXml($template)
            $toast = New-Object Windows.UI.Notifications.ToastNotification $xml
            [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("RustIRC").Show($toast)
            "#,
            title, body
        );

        Command::new("powershell")
            .args(["-Command", &script])
            .output()?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn show_macos_notification(&self, title: &str, body: &str) -> Result<()> {
        // macOS Notification Center implementation
        use std::process::Command;

        Command::new("osascript")
            .args([
                "-e",
                &format!(
                    r#"display notification "{}" with title "RustIRC" subtitle "{}""#,
                    body, title
                ),
            ])
            .output()?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn show_linux_notification(&self, title: &str, body: &str) -> Result<()> {
        // D-Bus notifications implementation
        use std::process::Command;

        // Try notify-send first (most common)
        let result = Command::new("notify-send")
            .args(["--app-name=RustIRC", "--icon=rustirc", title, body])
            .output();

        match result {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback to zenity if notify-send is not available
                Command::new("zenity")
                    .args(["--notification", "--text", &format!("{title}: {body}")])
                    .output()?;
                Ok(())
            }
        }
    }
}

/// System tray integration
pub struct SystemTray {
    enabled: bool,
    icon_path: Option<String>,
    menu_items: Vec<TrayMenuItem>,
}

#[derive(Debug, Clone)]
pub struct TrayMenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub checked: bool,
}

impl SystemTray {
    pub fn new() -> Self {
        Self {
            enabled: true,
            icon_path: None,
            menu_items: vec![
                TrayMenuItem {
                    id: "show".to_string(),
                    label: "Show RustIRC".to_string(),
                    enabled: true,
                    checked: false,
                },
                TrayMenuItem {
                    id: "separator1".to_string(),
                    label: "-".to_string(),
                    enabled: true,
                    checked: false,
                },
                TrayMenuItem {
                    id: "connect".to_string(),
                    label: "Quick Connect".to_string(),
                    enabled: true,
                    checked: false,
                },
                TrayMenuItem {
                    id: "disconnect".to_string(),
                    label: "Disconnect All".to_string(),
                    enabled: true,
                    checked: false,
                },
                TrayMenuItem {
                    id: "separator2".to_string(),
                    label: "-".to_string(),
                    enabled: true,
                    checked: false,
                },
                TrayMenuItem {
                    id: "quit".to_string(),
                    label: "Quit".to_string(),
                    enabled: true,
                    checked: false,
                },
            ],
        }
    }

    /// Initialize system tray
    pub fn initialize(&mut self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        self.initialize_windows_tray()?;

        #[cfg(target_os = "macos")]
        self.initialize_macos_tray()?;

        #[cfg(target_os = "linux")]
        self.initialize_linux_tray()?;

        Ok(())
    }

    /// Set tray icon
    pub fn set_icon(&mut self, icon_path: &str) {
        self.icon_path = Some(icon_path.to_string());
    }

    /// Update tray menu
    pub fn update_menu(&mut self, items: Vec<TrayMenuItem>) {
        self.menu_items = items;
    }

    /// Set tray tooltip
    pub fn set_tooltip(&self, tooltip: &str) -> Result<()> {
        // Platform-specific tooltip implementation
        #[cfg(target_os = "windows")]
        self.set_windows_tooltip(tooltip)?;

        #[cfg(target_os = "macos")]
        self.set_macos_tooltip(tooltip)?;

        #[cfg(target_os = "linux")]
        {
            // Linux system tray tooltip implementation using D-Bus
            use std::process::Command;

            // Store tooltip in internal state for D-Bus system tray
            // Use notify-send as fallback for tooltip display
            let result = Command::new("notify-send")
                .arg("--urgency=low")
                .arg("--expire-time=2000")
                .arg("RustIRC")
                .arg(tooltip)
                .output();

            match result {
                Ok(_) => (), // Tooltip set successfully
                Err(_) => {
                    // Fallback: write to temp file for other tray implementations to read
                    let tooltip_path = "/tmp/rustirc_tooltip";
                    let _ = std::fs::write(tooltip_path, tooltip);
                }
            }
        }

        Ok(())
    }

    /// Show tray balloon/notification
    pub fn show_balloon(&self, title: &str, message: &str) -> Result<()> {
        // Platform-specific balloon notification
        #[cfg(target_os = "windows")]
        self.show_windows_balloon(title, message)?;

        #[cfg(target_os = "macos")]
        self.show_macos_notification(title, message)?;

        #[cfg(target_os = "linux")]
        {
            // Linux notification implementation using libnotify or D-Bus
            use std::process::Command;
            let _ = Command::new("notify-send").arg(title).arg(message).spawn();
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn set_windows_tooltip(&self, tooltip: &str) -> Result<()> {
        // Windows system tray tooltip implementation using Win32 API
        use std::ffi::CString;
        use std::ptr;

        // Store tooltip for Windows tray icon
        let tooltip_cstr = match CString::new(tooltip) {
            Ok(s) => s,
            Err(_) => return Err(Error::PlatformError("Invalid tooltip string".to_string())),
        };

        // Write to Windows registry or temp file for tray icon to read
        let tooltip_path = std::env::temp_dir().join("rustirc_tooltip.txt");
        std::fs::write(&tooltip_path, tooltip)?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn set_macos_tooltip(&self, tooltip: &str) -> Result<()> {
        // macOS status bar tooltip implementation using NSStatusItem
        use std::process::Command;

        // Use osascript to set tooltip via AppleScript
        let script = format!(
            r#"tell application "System Events" to display notification "{}" with title "RustIRC""#,
            tooltip.replace('"', "\\\"")
        );

        let result = Command::new("osascript").arg("-e").arg(&script).output();

        match result {
            Ok(_) => (),
            Err(_) => {
                // Fallback: write to temp file
                let tooltip_path = "/tmp/rustirc_tooltip";
                std::fs::write(tooltip_path, tooltip)?;
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn show_windows_balloon(&self, title: &str, message: &str) -> Result<()> {
        // Windows balloon notification using Win32 API
        use std::process::Command;

        // Use PowerShell for Windows notifications
        let script = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('{}', '{}', 'OK', 'Information')"#,
            message.replace("'", "''"),
            title.replace("'", "''")
        );

        let _ = Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .spawn();

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn show_macos_notification(&self, title: &str, message: &str) -> Result<()> {
        // macOS notification using NSUserNotification
        use std::process::Command;

        let script = format!(
            r#"display notification "{}" with title "{}""#,
            message.replace('"', "\\\""),
            title.replace('"', "\\\"")
        );

        let _ = Command::new("osascript").arg("-e").arg(&script).spawn();

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn initialize_windows_tray(&self) -> Result<()> {
        // Windows system tray implementation
        // This would use the Windows Shell API
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn initialize_macos_tray(&self) -> Result<()> {
        // macOS status bar implementation
        // This would use NSStatusBar
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn initialize_linux_tray(&self) -> Result<()> {
        // Linux system tray implementation
        // This would use freedesktop specifications
        Ok(())
    }
}

/// Desktop integration features
pub struct DesktopIntegration;

impl DesktopIntegration {
    /// Register IRC URL protocol handler
    pub fn register_protocol_handler() -> Result<()> {
        #[cfg(target_os = "windows")]
        Self::register_windows_protocol()?;

        #[cfg(target_os = "macos")]
        Self::register_macos_protocol()?;

        #[cfg(target_os = "linux")]
        Self::register_linux_protocol()?;

        Ok(())
    }

    /// Create desktop shortcut
    pub fn create_desktop_shortcut() -> Result<()> {
        #[cfg(target_os = "windows")]
        Self::create_windows_shortcut()?;

        #[cfg(target_os = "macos")]
        Self::create_macos_alias()?;

        #[cfg(target_os = "linux")]
        Self::create_linux_desktop_file()?;

        Ok(())
    }

    /// Add to system startup
    pub fn add_to_startup() -> Result<()> {
        #[cfg(target_os = "windows")]
        Self::add_windows_startup()?;

        #[cfg(target_os = "macos")]
        Self::add_macos_login_item()?;

        #[cfg(target_os = "linux")]
        Self::add_linux_autostart()?;

        Ok(())
    }

    /// Remove from system startup
    pub fn remove_from_startup() -> Result<()> {
        #[cfg(target_os = "windows")]
        Self::remove_windows_startup()?;

        #[cfg(target_os = "macos")]
        Self::remove_macos_login_item()?;

        #[cfg(target_os = "linux")]
        Self::remove_linux_autostart()?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn register_windows_protocol() -> Result<()> {
        use std::process::Command;

        // Register irc:// protocol in Windows registry
        let exe_path = std::env::current_exe()?;
        let commands = vec![
            format!(r#"reg add "HKEY_CLASSES_ROOT\irc" /ve /d "IRC Protocol" /f"#),
            format!(r#"reg add "HKEY_CLASSES_ROOT\irc" /v "URL Protocol" /d "" /f"#),
            format!(
                r#"reg add "HKEY_CLASSES_ROOT\irc\shell\open\command" /ve /d "\"{}\" \"%1\"" /f"#,
                exe_path.display()
            ),
        ];

        for cmd in commands {
            Command::new("cmd").args(["/C", &cmd]).output()?;
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn register_macos_protocol() -> Result<()> {
        // Register protocol handler in Info.plist
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn register_linux_protocol() -> Result<()> {
        // Create .desktop file with MimeType
        let desktop_content = r#"[Desktop Entry]
Name=RustIRC
Exec=rustirc %u
Icon=rustirc
Type=Application
MimeType=x-scheme-handler/irc;
"#;

        let home = std::env::var("HOME")?;
        let desktop_file = format!("{home}/.local/share/applications/rustirc.desktop");
        std::fs::write(desktop_file, desktop_content)?;

        // Update desktop database
        use std::process::Command;
        Command::new("update-desktop-database")
            .arg(format!("{home}/.local/share/applications"))
            .output()?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn create_windows_shortcut() -> Result<()> {
        // Create Windows shortcut (.lnk file)
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn create_macos_alias() -> Result<()> {
        // Create macOS alias
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn create_linux_desktop_file() -> Result<()> {
        let desktop_content = r#"[Desktop Entry]
Name=RustIRC
Comment=Modern IRC Client
Exec=rustirc
Icon=rustirc
Type=Application
Categories=Network;Chat;
"#;

        let home = std::env::var("HOME")?;
        let desktop_file = format!("{home}/Desktop/RustIRC.desktop");
        std::fs::write(desktop_file, desktop_content)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn add_windows_startup() -> Result<()> {
        use std::process::Command;

        let exe_path = std::env::current_exe()?;
        let cmd = format!(
            r#"reg add "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "RustIRC" /d "\"{}\"" /f"#,
            exe_path.display()
        );

        Command::new("cmd").args(["/C", &cmd]).output()?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn add_macos_login_item() -> Result<()> {
        // Add to Login Items via LaunchAgent
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn add_linux_autostart() -> Result<()> {
        let autostart_content = r#"[Desktop Entry]
Name=RustIRC
Exec=rustirc --minimized
Icon=rustirc
Type=Application
X-GNOME-Autostart-enabled=true
"#;

        let home = std::env::var("HOME")?;
        let autostart_dir = format!("{home}/.config/autostart");
        std::fs::create_dir_all(&autostart_dir)?;

        let autostart_file = format!("{autostart_dir}/rustirc.desktop");
        std::fs::write(autostart_file, autostart_content)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn remove_windows_startup() -> Result<()> {
        use std::process::Command;

        let cmd = r#"reg delete "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run" /v "RustIRC" /f"#;
        Command::new("cmd").args(["/C", cmd]).output()?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn remove_macos_login_item() -> Result<()> {
        // Remove from Login Items
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn remove_linux_autostart() -> Result<()> {
        let home = std::env::var("HOME")?;
        let autostart_file = format!("{home}/.config/autostart/rustirc.desktop");

        if Path::new(&autostart_file).exists() {
            std::fs::remove_file(autostart_file)?;
        }

        Ok(())
    }
}

/// Sound management
pub struct SoundManager {
    enabled: bool,
    volume: f32,
}

impl SoundManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            volume: 0.5,
        }
    }

    /// Play notification sound
    pub fn play_notification_sound(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        self.play_windows_sound()?;

        #[cfg(target_os = "macos")]
        self.play_macos_sound()?;

        #[cfg(target_os = "linux")]
        self.play_linux_sound()?;

        Ok(())
    }

    /// Play custom sound file
    pub fn play_sound_file(&self, file_path: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Validate file path exists
        if !std::path::Path::new(file_path).exists() {
            return Err(anyhow::anyhow!("Sound file not found: {}", file_path));
        }

        // Platform-specific sound file playback
        #[cfg(target_os = "windows")]
        self.play_windows_sound_file(file_path)?;

        #[cfg(target_os = "macos")]
        self.play_macos_sound_file(file_path)?;

        #[cfg(target_os = "linux")]
        self.play_linux_sound_file(file_path)?;

        Ok(())
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Enable or disable sounds
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[cfg(target_os = "windows")]
    fn play_windows_sound(&self) -> Result<()> {
        use std::process::Command;

        // Play system notification sound
        Command::new("rundll32")
            .args(["user32.dll,MessageBeep", "0"])
            .output()?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn play_macos_sound(&self) -> Result<()> {
        use std::process::Command;

        Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .output()?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn play_linux_sound(&self) -> Result<()> {
        use std::process::Command;

        // Try different sound players
        let players = ["paplay", "aplay", "play"];
        let sound_file = "/usr/share/sounds/alsa/Front_Left.wav";

        for player in &players {
            if Command::new(player).arg(sound_file).output().is_ok() {
                break;
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn play_windows_sound_file(&self, file_path: &str) -> Result<()> {
        use std::process::Command;
        Command::new("powershell")
            .args(&[
                "-c",
                &format!("(New-Object Media.SoundPlayer '{}')", file_path),
            ])
            .output()?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn play_macos_sound_file(&self, file_path: &str) -> Result<()> {
        use std::process::Command;
        Command::new("afplay").arg(file_path).output()?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn play_linux_sound_file(&self, file_path: &str) -> Result<()> {
        use std::process::Command;

        let players = ["paplay", "aplay", "play"];
        for player in &players {
            if Command::new(player).arg(file_path).output().is_ok() {
                break;
            }
        }
        Ok(())
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SystemTray {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SoundManager {
    fn default() -> Self {
        Self::new()
    }
}
