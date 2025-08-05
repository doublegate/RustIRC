# Phase 7: Release and Distribution

**Duration**: 2+ weeks (ongoing)  
**Goal**: Successfully launch RustIRC 1.0 and establish distribution channels

## Overview

Phase 7 marks the transition from development to production release. This phase focuses on packaging RustIRC for all supported platforms, establishing distribution channels, launching the project publicly, and setting up infrastructure for ongoing maintenance and community support.

## Objectives

1. Create platform-specific packages
2. Set up distribution channels
3. Launch marketing and outreach
4. Establish community infrastructure
5. Plan post-release maintenance
6. Monitor adoption and feedback

## Packaging and Distribution

### Platform Packages

#### Windows Packaging
```rust
// build-scripts/windows/build.rs
use std::process::Command;

fn build_windows_installer() -> Result<()> {
    // Build release binary
    Command::new("cargo")
        .args(&["build", "--release", "--target", "x86_64-pc-windows-msvc"])
        .status()?;
    
    // Sign the binary
    sign_binary("target/x86_64-pc-windows-msvc/release/rustirc.exe")?;
    
    // Create installer with WiX or NSIS
    create_msi_installer()?;
    
    // Sign the installer
    sign_binary("dist/RustIRC-Setup.msi")?;
    
    Ok(())
}

fn create_msi_installer() -> Result<()> {
    // WiX configuration
    let wix_config = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
        <Product Id="*" Name="RustIRC" Version="1.0.0" 
                 Manufacturer="RustIRC Contributors" Language="1033">
            <Package InstallerVersion="200" Compressed="yes" />
            <Media Id="1" Cabinet="RustIRC.cab" EmbedCab="yes" />
            
            <Directory Id="TARGETDIR" Name="SourceDir">
                <Directory Id="ProgramFilesFolder">
                    <Directory Id="INSTALLFOLDER" Name="RustIRC" />
                </Directory>
            </Directory>
            
            <Feature Id="MainApplication" Title="RustIRC" Level="1">
                <ComponentRef Id="MainExecutable" />
                <ComponentRef Id="StartMenuShortcut" />
            </Feature>
        </Product>
    </Wix>
    "#;
    
    // Build MSI
    Command::new("candle").arg("rustirc.wxs").status()?;
    Command::new("light").arg("rustirc.wixobj").status()?;
    
    Ok(())
}
```

#### macOS Packaging
```bash
#!/bin/bash
# build-scripts/macos/build.sh

# Build universal binary
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create \
    target/x86_64-apple-darwin/release/rustirc \
    target/aarch64-apple-darwin/release/rustirc \
    -output target/release/rustirc-universal

# Create app bundle
mkdir -p RustIRC.app/Contents/{MacOS,Resources}
cp target/release/rustirc-universal RustIRC.app/Contents/MacOS/rustirc
cp assets/Info.plist RustIRC.app/Contents/
cp assets/icon.icns RustIRC.app/Contents/Resources/

# Sign the app
codesign --deep --force --sign "Developer ID Application: Your Name" RustIRC.app

# Create DMG
create-dmg \
    --volname "RustIRC" \
    --window-size 600 400 \
    --icon-size 100 \
    --icon "RustIRC.app" 175 120 \
    --hide-extension "RustIRC.app" \
    --app-drop-link 425 120 \
    "RustIRC-1.0.0.dmg" \
    "RustIRC.app"

# Sign the DMG
codesign --sign "Developer ID Application: Your Name" RustIRC-1.0.0.dmg

# Notarize
xcrun altool --notarize-app \
    --primary-bundle-id "com.rustirc.app" \
    --file RustIRC-1.0.0.dmg
```

#### Linux Packaging
```bash
#!/bin/bash
# build-scripts/linux/build.sh

# Build for multiple architectures
for arch in x86_64 aarch64; do
    cargo build --release --target ${arch}-unknown-linux-gnu
done

# Create Debian package
mkdir -p debian/usr/bin
mkdir -p debian/usr/share/applications
mkdir -p debian/usr/share/icons/hicolor/256x256/apps
mkdir -p debian/DEBIAN

cp target/x86_64-unknown-linux-gnu/release/rustirc debian/usr/bin/
cp assets/rustirc.desktop debian/usr/share/applications/
cp assets/icon.png debian/usr/share/icons/hicolor/256x256/apps/rustirc.png

cat > debian/DEBIAN/control << EOF
Package: rustirc
Version: 1.0.0
Architecture: amd64
Maintainer: RustIRC Contributors <rustirc@example.com>
Description: Modern IRC client written in Rust
Depends: libc6, libssl1.1
EOF

dpkg-deb --build debian rustirc-1.0.0-amd64.deb

# Create Flatpak
flatpak-builder --repo=repo build-dir com.rustirc.RustIRC.json
flatpak build-bundle repo rustirc-1.0.0.flatpak com.rustirc.RustIRC
```

### Package Distribution

#### GitHub Releases
```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: windows-latest
            artifact: RustIRC-Setup.msi
          - os: macos-latest
            artifact: RustIRC-*.dmg
          - os: ubuntu-latest
            artifact: rustirc-*.deb
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Build
      run: ./build-scripts/${{ matrix.os }}/build.sh
    
    - name: Upload Release Asset
      uses: actions/upload-release-asset@v1
      with:
        upload_url: ${{ github.event.release.upload_url }}
        asset_path: ./dist/${{ matrix.artifact }}
        asset_name: ${{ matrix.artifact }}
```

#### Package Managers

##### Homebrew Formula
```ruby
# Formula/rustirc.rb
class Rustirc < Formula
  desc "Modern IRC client written in Rust"
  homepage "https://rustirc.org"
  url "https://github.com/rustirc/rustirc/archive/v1.0.0.tar.gz"
  sha256 "..."
  license "GPL-3.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "RustIRC #{version}", shell_output("#{bin}/rustirc --version")
  end
end
```

##### AUR Package
```bash
# PKGBUILD
pkgname=rustirc
pkgver=1.0.0
pkgrel=1
pkgdesc="Modern IRC client written in Rust"
arch=('x86_64' 'aarch64')
url="https://rustirc.org"
license=('GPL3')
depends=('gcc-libs')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/rustirc/rustirc/archive/v$pkgver.tar.gz")
sha256sums=('...')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/rustirc" "$pkgdir/usr/bin/rustirc"
  install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

## Launch Strategy

### Project Website
```html
<!-- index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>RustIRC - Modern IRC Client</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <header>
        <h1>RustIRC</h1>
        <p>A modern, safe, and extensible IRC client</p>
    </header>
    
    <section id="features">
        <h2>Features</h2>
        <ul>
            <li>🦀 Written in Rust for safety and performance</li>
            <li>🎨 Beautiful GUI and efficient TUI</li>
            <li>🔌 Powerful scripting with Lua</li>
            <li>🔒 Secure by default with TLS and SASL</li>
            <li>📁 DCC file transfers with resume</li>
            <li>🌐 Full IRCv3 support</li>
        </ul>
    </section>
    
    <section id="download">
        <h2>Download</h2>
        <div class="download-buttons">
            <a href="/download/windows" class="btn">Windows</a>
            <a href="/download/macos" class="btn">macOS</a>
            <a href="/download/linux" class="btn">Linux</a>
        </div>
    </section>
</body>
</html>
```

### Announcement Template
```markdown
# Announcing RustIRC 1.0

After months of development, we're excited to announce the release of RustIRC 1.0, 
a modern IRC client that combines the best features of mIRC, HexChat, and WeeChat 
while leveraging Rust's safety and performance.

## Why RustIRC?

With HexChat's discontinuation in early 2024, the IRC community needed a modern, 
actively maintained client. RustIRC fills this gap by providing:

- **Cross-platform support**: Native on Windows, macOS, and Linux
- **Dual interface**: Beautiful GUI with Iced, efficient TUI with ratatui  
- **Extensibility**: Powerful Lua scripting and binary plugins
- **Modern protocols**: Full IRCv3 support, DCC with resume, SASL auth
- **Security first**: Sandboxed scripts, TLS by default, secure auth

## Get Started

Download RustIRC from:
- Website: https://rustirc.org
- GitHub: https://github.com/rustirc/rustirc
- Package managers: brew, apt, yay, etc.

## Join the Community

- IRC: #rustirc on Libera.Chat
- GitHub: Report issues and contribute
- Discord/Matrix: For real-time discussion

We welcome contributions and feedback as we continue to improve RustIRC!
```

### Community Channels

#### IRC Presence
```
/msg ChanServ REGISTER #rustirc
/msg ChanServ SET #rustirc TOPICLOCK ON
/msg ChanServ SET #rustirc GUARD ON
/topic #rustirc RustIRC 1.0 Released! | https://rustirc.org | https://github.com/rustirc/rustirc
```

#### Social Media
```python
# scripts/announce.py
import tweepy
import praw

def announce_release():
    # Twitter
    twitter = tweepy.Client(bearer_token=TWITTER_TOKEN)
    twitter.create_tweet(
        text="🎉 RustIRC 1.0 is here! A modern IRC client written in Rust with "
             "powerful scripting, beautiful UI, and full IRCv3 support. "
             "Download: https://rustirc.org #rust #irc #opensource"
    )
    
    # Reddit
    reddit = praw.Reddit(client_id=REDDIT_ID, client_secret=REDDIT_SECRET)
    reddit.subreddit("rust").submit(
        title="RustIRC 1.0 Released - Modern IRC Client in Rust",
        url="https://rustirc.org"
    )
    
    # Hacker News
    # Manual submission recommended
```

## Post-Release Maintenance

### Version Strategy
```toml
# Version numbering
# 1.0.0 - Initial release
# 1.0.x - Bug fixes only
# 1.1.0 - New features
# 2.0.0 - Breaking changes

[workspace.package]
version = "1.0.0"
```

### Release Cycle
```markdown
## Release Schedule

### Patch Releases (1.0.x)
- As needed for critical bugs
- Security fixes ASAP
- No new features

### Minor Releases (1.x.0)
- Every 3 months
- New features
- Backward compatible

### Major Releases (x.0.0)
- Yearly evaluation
- Breaking changes allowed
- Migration guide required
```

### Support Policy
```markdown
## Support Policy

### Version Support
- Latest stable: Full support
- Previous minor: Security fixes only
- Older versions: Community support

### Security
- Report to: security@rustirc.org
- GPG key: [public key]
- Response time: 48 hours

### Bug Reports
- GitHub Issues preferred
- Include version, OS, steps to reproduce
- Crash dumps appreciated
```

## Community Building

### Documentation Hub
```nginx
# nginx.conf
server {
    server_name docs.rustirc.org;
    
    location / {
        root /var/www/docs;
    }
    
    location /api {
        root /var/www/api-docs;
    }
    
    location /scripting {
        root /var/www/scripting-guide;
    }
}
```

### Script Repository
```rust
// scripts.rustirc.org backend
#[derive(Serialize)]
struct Script {
    id: String,
    name: String,
    description: String,
    author: String,
    version: String,
    downloads: u64,
    rating: f32,
    source_url: String,
}

#[get("/api/scripts")]
async fn list_scripts(db: &State<Database>) -> Json<Vec<Script>> {
    let scripts = db.get_approved_scripts().await?;
    Json(scripts)
}

#[post("/api/scripts/<id>/download")]
async fn download_script(id: String, db: &State<Database>) -> Result<String> {
    db.increment_downloads(&id).await?;
    Ok(db.get_script_content(&id).await?)
}
```

## Metrics and Monitoring

### Usage Analytics
```rust
// Anonymous telemetry (opt-in)
#[derive(Serialize)]
struct TelemetryData {
    version: String,
    os: String,
    ui_mode: String, // gui/tui
    feature_usage: HashMap<String, u64>,
}

async fn send_telemetry(data: TelemetryData) {
    if !settings.telemetry_enabled {
        return;
    }
    
    let client = reqwest::Client::new();
    let _ = client.post("https://telemetry.rustirc.org/v1/usage")
        .json(&data)
        .send()
        .await;
}
```

### Success Metrics
- Download counts per platform
- Active users (telemetry)
- GitHub stars/forks
- Issue resolution time
- Community growth
- Script repository activity

## Long-Term Roadmap

### Version 1.1 (3 months)
- [ ] Voice/video chat support (experimental)
- [ ] Enhanced script manager
- [ ] Performance improvements
- [ ] Additional themes

### Version 1.2 (6 months)
- [ ] Mobile companion app
- [ ] Cloud sync support
- [ ] Plugin marketplace
- [ ] Advanced search features

### Version 2.0 (1 year)
- [ ] Protocol bridges (Matrix, XMPP)
- [ ] Built-in bouncer mode
- [ ] Web UI option
- [ ] AI-powered features

## Success Criteria

Phase 7 is successful when:
- [ ] All platform packages available
- [ ] 1000+ downloads in first week
- [ ] Active community established
- [ ] Positive user feedback
- [ ] Major IRC networks aware
- [ ] Sustainable development pace

## Ongoing Tasks

Post-release maintenance continues indefinitely:
- Monitor issue tracker
- Respond to security reports
- Release patches as needed
- Engage with community
- Plan future features
- Maintain infrastructure