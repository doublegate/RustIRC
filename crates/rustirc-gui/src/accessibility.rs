//! Accessibility features for RustIRC GUI
//!
//! Provides screen reader support, keyboard navigation, and other
//! accessibility features to ensure RustIRC is usable by everyone.

use anyhow::Result;
use std::collections::HashMap;

/// Accessibility manager for the GUI
pub struct AccessibilityManager {
    enabled: bool,
    screen_reader_enabled: bool,
    high_contrast: bool,
    font_scale: f32,
    announcement_queue: Vec<String>,
}

impl AccessibilityManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            screen_reader_enabled: detect_screen_reader(),
            high_contrast: false,
            font_scale: 1.0,
            announcement_queue: Vec::new(),
        }
    }
    
    /// Announce text to screen readers
    pub fn announce(&mut self, text: &str) -> Result<()> {
        if !self.enabled || !self.screen_reader_enabled {
            return Ok(());
        }
        
        self.announcement_queue.push(text.to_string());
        
        #[cfg(target_os = "windows")]
        self.announce_windows(text)?;
        
        #[cfg(target_os = "macos")]
        self.announce_macos(text)?;
        
        #[cfg(target_os = "linux")]
        self.announce_linux(text)?;
        
        Ok(())
    }
    
    /// Announce new IRC message
    pub fn announce_message(&mut self, sender: &str, message: &str, channel: Option<&str>) -> Result<()> {
        let announcement = if let Some(channel) = channel {
            format!("Message in {}: {} says {}", channel, sender, message)
        } else {
            format!("Private message from {}: {}", sender, message)
        };
        
        self.announce(&announcement)
    }
    
    /// Announce channel join/part
    pub fn announce_channel_event(&mut self, nick: &str, channel: &str, event: ChannelEvent) -> Result<()> {
        let announcement = match event {
            ChannelEvent::Join => format!("{} joined {}", nick, channel),
            ChannelEvent::Part => format!("{} left {}", nick, channel),
            ChannelEvent::Quit => format!("{} quit", nick),
            ChannelEvent::Nick(new_nick) => format!("{} is now known as {}", nick, new_nick),
        };
        
        self.announce(&announcement)
    }
    
    /// Announce focus change
    pub fn announce_focus_change(&mut self, element: &str) -> Result<()> {
        let announcement = format!("Focus moved to {}", element);
        self.announce(&announcement)
    }
    
    /// Announce status change
    pub fn announce_status(&mut self, status: &str) -> Result<()> {
        self.announce(status)
    }
    
    /// Enable or disable accessibility features
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Enable or disable screen reader support
    pub fn set_screen_reader_enabled(&mut self, enabled: bool) {
        self.screen_reader_enabled = enabled;
    }
    
    /// Enable or disable high contrast mode
    pub fn set_high_contrast(&mut self, enabled: bool) {
        self.high_contrast = enabled;
    }
    
    /// Set font scaling factor
    pub fn set_font_scale(&mut self, scale: f32) {
        self.font_scale = scale.clamp(0.5, 3.0);
    }
    
    /// Get current font scale
    pub fn font_scale(&self) -> f32 {
        self.font_scale
    }
    
    /// Check if high contrast mode is enabled
    pub fn is_high_contrast(&self) -> bool {
        self.high_contrast
    }
    
    /// Check if screen reader is detected/enabled
    pub fn is_screen_reader_enabled(&self) -> bool {
        self.screen_reader_enabled
    }
    
    /// Clear announcement queue
    pub fn clear_announcements(&mut self) {
        self.announcement_queue.clear();
    }
    
    #[cfg(target_os = "windows")]
    fn announce_windows(&self, text: &str) -> Result<()> {
        use std::process::Command;
        
        // Use SAPI (Speech API) for announcements
        let script = format!(
            r#"
            Add-Type -AssemblyName System.Speech
            $synthesizer = New-Object System.Speech.Synthesis.SpeechSynthesizer
            $synthesizer.Speak("{}")
            "#,
            text.replace('"', "\"\"")
        );
        
        Command::new("powershell")
            .args(["-Command", &script])
            .output()?;
            
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    fn announce_macos(&self, text: &str) -> Result<()> {
        use std::process::Command;
        
        Command::new("say")
            .arg(text)
            .output()?;
            
        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    fn announce_linux(&self, text: &str) -> Result<()> {
        use std::process::Command;
        
        // Try espeak first, then festival
        let result = Command::new("espeak")
            .arg(text)
            .output();
            
        match result {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback to festival
                Command::new("festival")
                    .arg("--tts")
                    .arg(text)
                    .output()?;
                Ok(())
            }
        }
    }
}

/// Channel event types for announcements
#[derive(Debug, Clone)]
pub enum ChannelEvent {
    Join,
    Part,
    Quit,
    Nick(String),
}

/// Keyboard navigation helper
pub struct KeyboardNavigation {
    focus_ring: Vec<FocusableElement>,
    current_focus: usize,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct FocusableElement {
    pub id: String,
    pub element_type: ElementType,
    pub accessible_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Button,
    TextInput,
    List,
    Tree,
    Tab,
    Menu,
    MenuItem,
    MessageArea,
    UserList,
    ServerTree,
}

impl KeyboardNavigation {
    pub fn new() -> Self {
        Self {
            focus_ring: Vec::new(),
            current_focus: 0,
            enabled: true,
        }
    }
    
    /// Add element to focus ring
    pub fn add_element(&mut self, element: FocusableElement) {
        self.focus_ring.push(element);
    }
    
    /// Remove element from focus ring
    pub fn remove_element(&mut self, id: &str) {
        self.focus_ring.retain(|e| e.id != id);
        if self.current_focus >= self.focus_ring.len() && !self.focus_ring.is_empty() {
            self.current_focus = self.focus_ring.len() - 1;
        }
    }
    
    /// Move focus to next element
    pub fn focus_next(&mut self) -> Option<&FocusableElement> {
        if self.focus_ring.is_empty() || !self.enabled {
            return None;
        }
        
        self.current_focus = (self.current_focus + 1) % self.focus_ring.len();
        self.focus_ring.get(self.current_focus)
    }
    
    /// Move focus to previous element
    pub fn focus_previous(&mut self) -> Option<&FocusableElement> {
        if self.focus_ring.is_empty() || !self.enabled {
            return None;
        }
        
        self.current_focus = if self.current_focus == 0 {
            self.focus_ring.len() - 1
        } else {
            self.current_focus - 1
        };
        
        self.focus_ring.get(self.current_focus)
    }
    
    /// Focus specific element by ID
    pub fn focus_element(&mut self, id: &str) -> Option<&FocusableElement> {
        if let Some(index) = self.focus_ring.iter().position(|e| e.id == id) {
            self.current_focus = index;
            self.focus_ring.get(self.current_focus)
        } else {
            None
        }
    }
    
    /// Get currently focused element
    pub fn current_focus(&self) -> Option<&FocusableElement> {
        self.focus_ring.get(self.current_focus)
    }
    
    /// Enable or disable keyboard navigation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Clear all focusable elements
    pub fn clear(&mut self) {
        self.focus_ring.clear();
        self.current_focus = 0;
    }
}

/// ARIA (Accessible Rich Internet Applications) support
pub struct AriaSupport {
    labels: HashMap<String, String>,
    descriptions: HashMap<String, String>,
    roles: HashMap<String, AriaRole>,
    properties: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AriaRole {
    Application,
    Main,
    Navigation,
    Search,
    Region,
    Button,
    Link,
    Tab,
    TabPanel,
    TabList,
    Tree,
    TreeItem,
    List,
    ListItem,
    TextBox,
    Log,
    Status,
    Alert,
}

impl AriaSupport {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            descriptions: HashMap::new(),
            roles: HashMap::new(),
            properties: HashMap::new(),
        }
    }
    
    /// Set ARIA label for element
    pub fn set_label(&mut self, element_id: &str, label: &str) {
        self.labels.insert(element_id.to_string(), label.to_string());
    }
    
    /// Set ARIA description for element
    pub fn set_description(&mut self, element_id: &str, description: &str) {
        self.descriptions.insert(element_id.to_string(), description.to_string());
    }
    
    /// Set ARIA role for element
    pub fn set_role(&mut self, element_id: &str, role: AriaRole) {
        self.roles.insert(element_id.to_string(), role);
    }
    
    /// Set ARIA property
    pub fn set_property(&mut self, element_id: &str, property: &str, value: &str) {
        self.properties
            .entry(element_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(property.to_string(), value.to_string());
    }
    
    /// Get ARIA attributes for element
    pub fn get_attributes(&self, element_id: &str) -> AriaAttributes {
        AriaAttributes {
            label: self.labels.get(element_id).cloned(),
            description: self.descriptions.get(element_id).cloned(),
            role: self.roles.get(element_id).cloned(),
            properties: self.properties.get(element_id).cloned().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AriaAttributes {
    pub label: Option<String>,
    pub description: Option<String>,
    pub role: Option<AriaRole>,
    pub properties: HashMap<String, String>,
}

/// Screen reader detection
fn detect_screen_reader() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check for common Windows screen readers
        use std::process::Command;
        
        let screen_readers = ["NVDA", "JAWS", "WindowEyes", "ZoomText"];
        
        for reader in &screen_readers {
            if Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}.exe", reader)])
                .output()
                .map(|output| String::from_utf8_lossy(&output.stdout).contains(reader))
                .unwrap_or(false)
            {
                return true;
            }
        }
        
        false
    }
    
    #[cfg(target_os = "macos")]
    {
        // Check for VoiceOver
        use std::process::Command;
        
        Command::new("defaults")
            .args(["read", "com.apple.universalaccess", "voiceOverOnOffKey"])
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
    }
    
    #[cfg(target_os = "linux")]
    {
        // Check for Orca or other AT-SPI screen readers
        std::env::var("AT_SPI_BUS").is_ok() || 
        std::process::Command::new("pgrep")
            .arg("orca")
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
    }
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for KeyboardNavigation {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AriaSupport {
    fn default() -> Self {
        Self::new()
    }
}