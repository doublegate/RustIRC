# Dioxus.toml Configuration Reference

This document provides a comprehensive reference for configuring Dioxus v0.6 desktop applications using the `Dioxus.toml` file.

## Table of Contents
- [Application Configuration](#application-configuration)
- [Bundle Configuration](#bundle-configuration)
- [Web Configuration](#web-configuration)
- [Platform-Specific Configuration](#platform-specific-configuration)
- [Complete Example](#complete-example)

## Application Configuration

The `[application]` section contains core application settings:

```toml
[application]
# App name (MANDATORY)
name = "RustIRC"

# The Dioxus platform to default to (MANDATORY)
# Options: "web", "desktop", "mobile", "ssr"
default_platform = "desktop"

# Build & serve output path
out_dir = "dist"

# The static resource path
asset_dir = "assets"

# The sub package in the workspace to build by default
sub_package = "my-crate"
```

## Bundle Configuration

The `[bundle]` section controls application bundling using tauri-bundler:

```toml
[bundle]
# Unique identifier for your application (e.g., com.company.app)
identifier = "com.rustirc.client"

# Publisher name
publisher = "RustIRC Team"

# Icon files (must be square, specific pixel dimensions)
# macOS requires .icns, Windows requires .ico, Linux requires .png
icon = [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico"
]

# Additional files to include in the bundle (supports globs)
resources = ["assets/**/*.png", "config/*.toml"]

# Copyright information
copyright = "Copyright (c) 2025 RustIRC Team"

# Application category (must be from predefined list)
# Options: Business, DeveloperTool, Education, Entertainment, Finance, Game,
# GraphicsAndDesign, HealthcareAndFitness, Lifestyle, Medical, Music, News,
# Photography, Productivity, Reference, SocialNetworking, Sports, Travel,
# Utility, Video, Weather
category = "DeveloperTool"

# Brief description
short_description = "Modern IRC client with React-like UI"

# Detailed description
long_description = """
RustIRC is a modern IRC client built with Rust and Dioxus v0.6,
featuring a React-like component architecture, Virtual DOM,
and comprehensive IRC protocol support including IRCv3 extensions.
"""

# External sidecar binaries to include
external_bin = ["./sidecar-app"]
```

## Web Configuration

Even for desktop-only apps, some web configuration is required:

```toml
[web.app]
# HTML title tag content
title = "RustIRC - Modern IRC Client"

# Base path for serving the application
base_path = "/"

[web.watcher]
# Regenerate index.html when triggered
reload_html = true

# Directories to monitor for changes
watch_path = ["src", "assets"]

# Serve root page for 404s (required for client-side routing)
index_on_404 = true

[web.resource]
# CSS files to include
style = ["./assets/style.css"]

# JavaScript files to include
script = []

[web.resource.dev]
# Development-only resources
style = []
script = []

# Proxy configuration for backend API
[[web.proxy]]
backend = "http://localhost:8000/api/"
```

## Platform-Specific Configuration

### Linux (Debian/Ubuntu)

```toml
[bundle.linux]
# Debian package dependencies
deb_depends = ["libwebkit2gtk-4.1-0", "libgtk-3-0"]

# RPM package dependencies
rpm_depends = ["webkit2gtk4.1", "gtk3"]

# Custom files to add (maps debian path to local path)
files = {}

# Debian-specific settings
section = "net"
priority = "optional"
```

### macOS

```toml
[bundle.macos]
# Frameworks to include
frameworks = []

# Minimum macOS version
minimum_system_version = "10.15"

# License file path
license = "./LICENSE"

# Code signing identity
signing_identity = "Developer ID Application: Your Name"

# Provider short name
provider_short_name = "YourCompany"

# Entitlements file
entitlements = "entitlements.plist"

# Enable hardened runtime
hardened_runtime = false

# Include debug info
include_debug_info = false

# Custom files to add
files = {}
```

### Windows

```toml
[bundle.windows]
# WebView2 installation mode
# Options: DownloadBootstrapper, EmbedBootstrapper, OfflineInstaller, FixedRuntime, Skip
webview_install_mode = { DownloadBootstrapper = { silent = true } }

# WiX installer configuration
wix = {
  version = "3",
  language = [["en-US", "wix/locales/en-US.wxl"]],
  fragment_paths = [],
  component_group_refs = [],
  component_refs = [],
  feature_group_refs = [],
  feature_refs = [],
  merge_refs = [],
  skip_webview_install = false,
  enable_elevated_update_task = false,
  fips_compliant = false
}

# Code signing settings
digest_algorithm = "sha-256"
certificate_thumbprint = "A1B2C3D4E5F6..."
timestamp_url = "http://timestamp.digicert.com"

# Time stamping protocol
tsp = false

# Allow downgrades in installer
allow_downgrades = true

# System tray icon path
icon_path = "assets/icon.ico"

# Custom files to add
files = {}
```

## Complete Example

Here's a complete `Dioxus.toml` for a desktop IRC client:

```toml
[application]
name = "RustIRC"
default_platform = "desktop"
out_dir = "dist"
asset_dir = "assets"

[application.tools]
# Custom tools for development
tailwind = "npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch=always"

[web.app]
title = "RustIRC - Modern IRC Client"
base_path = "/"

[web.watcher]
reload_html = true
watch_path = ["src", "assets"]
index_on_404 = true

[web.resource]
style = ["assets/style.css"]
script = []

[web.resource.dev]
style = []
script = []

[bundle]
identifier = "com.rustirc.client"
publisher = "RustIRC Team"
icon = ["images/rustirc-logo.png"]
resources = ["assets"]
copyright = "Copyright (c) 2025 RustIRC Team"
category = "DeveloperTool"
short_description = "Modern IRC client with React-like UI"
long_description = """
RustIRC is a modern IRC client built with Rust and Dioxus v0.6,
featuring a React-like component architecture, Virtual DOM,
and comprehensive IRC protocol support including IRCv3 extensions.
"""

[bundle.linux]
deb_depends = ["libwebkit2gtk-4.1-0", "libgtk-3-0"]
rpm_depends = ["webkit2gtk4.1", "gtk3"]
files = {}

[bundle.macos]
frameworks = []
minimum_system_version = "10.15"
hardened_runtime = false
include_debug_info = false
files = {}

[bundle.windows]
webview_install_mode = { DownloadBootstrapper = { silent = true } }
wix = {
  version = "3",
  language = [["en-US", "wix/locales/en-US.wxl"]],
  fragment_paths = [],
  component_group_refs = [],
  component_refs = [],
  feature_group_refs = [],
  feature_refs = [],
  merge_refs = [],
  skip_webview_install = false,
  enable_elevated_update_task = false,
  fips_compliant = false
}
tsp = false
allow_downgrades = true
files = {}
```

## Notes

1. **Mandatory Fields**: The `name` and `default_platform` fields in `[application]` are required.
2. **Platform Icons**: Icons must be square with specific dimensions (16, 24, 32, 64, or 256 pixels).
3. **Categories**: Must use predefined category values from the list above.
4. **WebView2**: Windows apps require WebView2 runtime configuration.
5. **Development Server**: Use `dx serve --platform desktop` to test with hot reload.
6. **Bundling**: Use `dx bundle --release` to create distributable packages.