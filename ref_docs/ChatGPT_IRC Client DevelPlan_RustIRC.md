# IRC Client Development Plan: RustIRC

## Introduction

This document outlines a comprehensive plan for developing a modern IRC client named **RustIRC** using the Rust programming language. RustIRC aims to combine the best features from established IRC clients—mIRC (powerful scripting and customization), HexChat (user-friendly GUI and plugin support), and WeeChat (efficiency, scriptability, and buffer management)—while ensuring full compatibility with IRC standards, including IRCv3 extensions. The client will support essential protocols like DCC (Direct Client-to-Client) for file transfers and chats, SASL (Simple Authentication and Security Layer) mechanisms for secure authentication, and cross-platform operation on Linux, macOS, and Windows 10+. Rust is chosen for its safety, performance, and concurrency features, which are ideal for handling network I/O, multi-threaded operations (e.g. multiple server connections), and memory management without common pitfalls like buffer overflows. Development will prioritize modularity, extensibility, and user privacy, with a focus on open-source principles.

**Background:** In planning RustIRC, we reviewed existing clients and community feedback to identify valuable features and shortcomings:

* **mIRC (Windows-only, proprietary):** First released in 1995, mIRC became one of the most popular IRC clients for Windows. It is renowned for its integrated scripting language that can completely alter the client's behavior and appearance, plus built-in file sharing via DCC with a file server. This powerful event-driven scripting (mIRC scripting language, or mSL) enables extensive customization and automation. However, mIRC is closed-source **shareware** (requiring paid registration after a 30-day trial) and officially available only on Windows. Its Windows-centric design and outdated UI model (MDI interface reminiscent of 90s software) limit its accessibility on other platforms. Despite these drawbacks, many long-time users praise mIRC’s robust feature set and familiarity – some even continue running it on Linux/macOS via Wine or compatibility layers. For example, power users note that with the right add-ons (e.g. download filters), mIRC can outperform newer clients for tasks like automated file downloads.

* **HexChat (cross-platform, open-source):** HexChat is an IRC client forked from XChat, designed to be free on all platforms (unlike XChat’s Windows shareware model). It features a convenient GUI with either tabbed or tree-style channel interface, support for multiple simultaneous server connections, and extensive configuration options. HexChat runs natively on Windows and Unix-like systems, and was widely available via Linux distribution repositories. It is highly scriptable as well – users can extend HexChat with plugins or scripts in languages like Lua, Python, and Perl. Important capabilities such as SASL authentication, proxies, spellcheck, and DCC file transfers are supported out-of-the-box. HexChat gained popularity for being beginner-friendly yet powerful for advanced users. A user in 2021 remarked that while mIRC’s interface remained a personal favorite, “HexChat is probably the best nowadays” for a modern IRC client. The primary downside of HexChat has been its stagnant development in recent years; the project was discontinued in early 2024 due to lack of maintainers. This has led to community concerns about unaddressed bugs or future compatibility. Additionally, some experienced mIRC users found minor feature gaps during migration – for instance, HexChat’s default install lacked certain quick-access operator commands or the window-splitting flexibility found in mIRC’s UI (requiring additional plugins or workarounds). Overall, HexChat set a strong baseline (free, cross-platform, extensible), and RustIRC should build on that foundation while updating and optimizing where possible.

* **WeeChat (cross-platform, open-source):** WeeChat (Wee Enhanced Environment for Chat) is a fast, lightweight client known for its modular architecture and scriptability. It is primarily a terminal-based (ncurses) application, which makes it extremely efficient and popular among power users who appreciate keyboard-driven interfaces. WeeChat’s feature list is extensive: the interface can be split into multiple windows/panes to view several channels simultaneously, and it supports scripting in a multitude of languages (Perl, Python, Ruby, Lua, Tcl, Scheme/Guile, JavaScript, PHP) via its plugin API. It also includes advanced features like smart filtering of messages, 256-color support, and a relay plugin that allows remote GUI or web clients to attach. Users often praise WeeChat for being “lightweight, extensible, stable, customizable” and keeping up with the latest IRC protocol specs. Indeed, WeeChat has rapidly adopted IRCv3 extensions and other modern improvements. However, WeeChat’s console-based UX can be intimidating for newcomers who are accustomed to graphical interfaces – as one summary puts it, WeeChat **has no official GUI** and “might be challenging to use for those accustomed to graphical interfaces”. (Projects like Glowing Bear provide a web GUI for WeeChat, but that adds complexity.) In practice, some users choose WeeChat on Linux for its power, but opt for GUI clients on Windows or macOS. For RustIRC, the lesson is to offer WeeChat’s depth and efficiency in a more accessible UI, and optionally provide a TUI for those who want it.

**RustIRC’s Vision:** **RustIRC will be a “best-of-breed” IRC client**, incorporating the strengths of mIRC, HexChat, and WeeChat while addressing their weaknesses. That means RustIRC will feature a modern, customizable GUI (inspired by HexChat’s approachable design) combined with the ability to run in a command-line mode (TUI) for efficiency and remote use (like WeeChat). It will include a powerful scripting and plugin system comparable to mIRC’s flexibility but with the safety and variety of languages seen in HexChat/WeeChat. Full standards compliance (IRCv3, TLS, SASL, etc.) is non-negotiable – RustIRC will support the latest protocol extensions out-of-the-box, a area where WeeChat has set a high bar. At the same time, RustIRC will avoid pitfalls such as platform lock-in, outdated code, or poor default usability. The client will be free, open-source, and actively maintained. New features and enhancements not seen in older clients will be introduced, such as a **built-in script/plugin manager** (for discovering and installing extensions on the fly, inspired by WeeChat’s `/script` tool), more robust security (e.g. optional end-to-end encryption for private chats), and native integration with each OS’s look-and-feel and notification systems. By learning from user feedback on existing clients and leveraging Rust’s capabilities, RustIRC is poised to deliver a superior IRC experience for both casual and power users.

### Project Goals

* **Feature Parity and Innovation**: Deliver all core functionality expected of a modern IRC client, blending mIRC’s scripting depth, HexChat’s intuitive interface, and WeeChat’s lightweight efficiency into a single cohesive application. Wherever possible, introduce improvements – for example, better default UI for channel management (addressing gaps noted in HexChat vs. mIRC), more flexible window layouts (multiple channels visible, akin to WeeChat’s splits or mIRC’s MDI), and innovative features that enhance usability without deviating from IRC’s principles. The goal is a client that longtime IRC users will find familiar and powerful, yet new users will find approachable.

* **Standards Compliance**: Fully adhere to the IRC protocol specifications (RFC 1459 and RFC 2812) and modern extensions. RustIRC will implement the latest IRCv3 features – e.g. capability negotiation (`CAP LS/REQ`), server-time and message tags, multiline and batch, away-notify, account-tag, `CHATHISTORY` playback – to ensure it can operate seamlessly on networks like Libera.Chat or EFnet. By supporting SASL authentication (Plain, External (cert-based), SCRAM-SHA-256, etc.) and TLS encryption (including support for IRCv3 STS for automatic secure upgrades), RustIRC will provide secure connectivity out-of-the-box. Compatibility with popular IRC daemons and services (NickServ, ChanServ conventions, etc.) will be tested thoroughly. The client will pass interoperability tests and avoid behaviors that break protocol compliance.

* **Cross-Platform Support**: Provide first-class native support for **Linux**, **macOS**, and **Windows 10+**. This includes proper GUI toolkits for each (e.g. using a cross-platform Rust UI library or per-OS adaptations) so that RustIRC feels at home on each system. Ensure the build and release process produces platform-specific packages: for Windows, an installer (`.msi` or `.exe`) and possibly Microsoft Store package; for macOS, a signed `.dmg` and Homebrew formula; for Linux, binaries or packages for major distro formats (Debian/Ubuntu `.deb`, Fedora/OpenSUSE `.rpm`, Arch Linux AUR). Consider also distributing via universal Linux package managers like **Flatpak** for convenience. The application’s features and performance should be consistent across platforms. Cross-platform consistency was a key advantage of HexChat (which was available on Windows and Unix and packaged by many distros), and RustIRC will continue that tradition. GUI elements will follow the conventions of each OS (e.g. using native notification systems, fonts, and shortcuts where applicable). Continuous integration will test builds on all three OS families to prevent regressions.

* **Security and Performance**: Leverage Rust’s strengths (memory safety, concurrency without data races) to create an IRC client that is both secure against common vulnerabilities and efficient in resource usage. RustIRC should handle large numbers of channels or high message throughput with low CPU and memory footprint, comparable or better than existing clients. Async networking (using Rust’s async/await, Tokio runtime) will keep the UI responsive even under heavy load or slow network conditions. All network communication will support SSL/TLS (using modern TLS libraries and certificate validation by default). We will include SASL authentication for secure login, and support proxy connections (SOCKS5/HTTP proxies) for user privacy. Any sensitive operations (like parsing untrusted IRC messages) must be robust against malformed data or injection (e.g. no crashing on a long message tag, no code execution from color codes, etc.). We will audit the client to ensure it doesn’t repeat the security issues of the past (for instance, mIRC’s scripting had incidents of abuse via unchecked `$decode`, which RustIRC’s scripting subsystem will avoid by sandboxing or safe defaults). Performance-wise, the goal is to comfortably handle joining 100+ channels across multiple networks without lag – something we will achieve via optimized data structures (e.g. using efficient mappings for user lists) and background tasks for logging, etc. Rust’s zero-cost abstractions and compiled performance should give us an edge over script-heavy clients.

* **Extensibility**: Design RustIRC to be extensible at multiple levels. Users should be able to customize the client’s behavior without modifying core code – through **scripting, plugins, and themes**. Inspired by mIRC’s powerful internal scripting and HexChat/WeeChat’s plugin APIs, we will include a built-in scripting engine (see Functional Requirements below) that allows writing scripts to respond to events (on message, on join, etc.), create custom commands, and even modify the UI. The plugin system will allow compiled extensions (likely in Rust or C) to be loaded at runtime, as well as high-level scripts in languages like Lua or Python. This extensibility ensures that advanced users and the community can tailor the client to their needs – whether that’s automating a channel moderation bot, integrating with other services (e.g. post IRC messages to a webhook), or adding support for new IRCv3 features in the future. We’ll provide documentation and examples to make writing plugins/scripts straightforward. Importantly, the architecture will keep core and plugins isolated (process or thread boundaries as needed) so a misbehaving plugin can’t easily crash the whole client. Configuration will be extensible too, with user-defined aliases, shortcuts, and themeing via config files. The end goal is an IRC client that, much like its predecessors, develops a rich ecosystem of user-contributed add-ons – but with modern language safety and an easier distribution mechanism (for example, a built-in plugin/script manager for discovery).

### Target Audience

* **Casual IRC Users**: People who join IRC for interest-based communities or support channels and want a simple, reliable chat client. For these users, RustIRC will offer sensible defaults, an easy setup (e.g. a startup wizard or a pre-loaded list of IRC networks to choose from), and a clean GUI that doesn’t require technical knowledge. Features like logging, notifications on mentions, and an inviting interface (light/dark themes, emoji support if possible) will cater to this group.

* **Power Users and IRC Enthusiasts**: Long-time IRC users, system administrators, open-source contributors on IRC, etc., who demand advanced functionality. These users often run multiple client instances or bouncers, write scripts, and connect to several networks at once. RustIRC should appeal to them by providing the depth of customization (scripting, plugin APIs, extensive configuration) and efficiency (keyboard shortcuts, optional text-mode UI, ability to handle many channels). Essentially, it should be attractive for someone currently using WeeChat, irssi, or even sticking with mIRC for its scripting – giving them reason to switch by offering comparable power in a modern, safe package.

* **Developers/Contributors**: As an open-source project, RustIRC will also target developers who might contribute code or plugins. The project should be approachable for Rust developers (clean modular code, clear CONTRIBUTING guidelines) and perhaps even educational for those wanting to learn network programming in Rust. By using a popular tech stack (Tokio, etc.), we make it easier for interested developers to join. In the long run, we want a community around RustIRC similar to that of existing clients (for example, HexChat had community plugins; WeeChat has script repositories). Thus, documentation and code structure will be geared toward maintainability and collaboration.

* **Cross-Platform Users**: Individuals who use multiple operating systems and want a consistent IRC experience. For example, a user might use Windows at work and Linux at home – RustIRC should feel and function the same (within OS conventions) on both. The client will also be suitable for those on less common platforms (we aim to support ARM Linux for devices like Raspberry Pi, possibly support running on \*BSDs, etc. via community). The target audience is essentially anyone who needs IRC in 2025 and beyond, from chat novices to IRC veterans.

### Assumptions and Constraints

* **Development Team & Knowledge**: We assume a small development team (even a single developer to start) with intermediate-to-advanced Rust knowledge. The plan is scoped such that one proficient Rust developer could create an MVP in a few months, with additional help needed for polishing cross-platform GUI aspects. We also assume familiarity with IRC protocol details within the team or access to those resources.

* **Timeline**: Target timeline is roughly 6–12 months for reaching a feature-complete 1.0 release, given full-time effort. This includes time for research, core development, testing, and iteration based on user feedback. We assume the ability to release incremental versions (alphas, betas) to gather feedback early. If the team is larger or community contributes, this timeline could shorten; if development is part-time, it may extend.

* **Budget/Resources**: The project is envisioned as open-source with minimal budget. We assume no commercial funding – thus relying on free tools (GitHub for code, CI, etc.), and existing open-source crates for functionality (networking, UI, etc.). We won’t have a budget for commercial UI libraries or paid cross-platform frameworks, which is why we prioritize Rust-native or open-source solutions (like the `iced` GUI crate or GTK). Testing infrastructure will use free CI services (like GitHub Actions) and volunteer testing on different OSes. One implication is that we won’t design features that require paid services or infrastructure (for example, no centralized server component except optional community-run things like update check or plugin registry, which can be done in a distributed way).

* **Technical Constraints**: Using Rust means we inherit some constraints such as needing to manage C FFI carefully for any integration (e.g. embedding Lua or interfacing with OS-specific APIs). We assume that all major requirements (network sockets, TLS, thread management, GUI) can be fulfilled with existing Rust crates or moderate C bindings. We will avoid reinventing the wheel where possible (e.g., use a proven IRC parsing crate if available, use existing crypto libraries for SASL SCRAM, etc.). Another constraint: the client must be performant on reasonably low-end hardware (we assume users might run this on older PCs or small Linux ARM boards). We won’t require GPU acceleration beyond what a typical system provides, although modern systems with OpenGL/Vulkan might be leveraged via GUI frameworks by default.

* **Scope Limitations**: Initially, we limit scope to desktop use – mobile IRC clients are out of scope (smartphones typically use different apps or a bouncer/web model). We assume users who need mobile will use a relay or bouncer with RustIRC (which could act as the always-on end), or simply use a separate mobile client. Another constraint is that RustIRC will focus purely on IRC and not attempt to natively support other chat protocols (no bloat of multi-protocol support like some IM clients). The idea is to do IRC very well. Where integration is desired (like using Slack or Matrix bridges), it will be via gateway servers or plugins, not built into the core. This keeps our domain focused.

With these assumptions, we proceed with a detailed requirements specification and a phased development plan.

## Requirements Specification

### Functional Requirements

1. **Core IRC Protocol Support**: RustIRC must implement all standard IRC commands and capabilities necessary for typical usage.

   * **Multi-Server Connections**: The client will allow connecting to multiple IRC servers/networks concurrently, each with its own set of channels and private chats. Users can add new server configurations (address, port, SSL, credentials) and connect/disconnect independently. Internally, this means handling separate connection states and message loops per server (leveraging async tasks or threads). Multi-server support was present in HexChat and WeeChat, and is expected by users.
   * **Channels & Messaging**: Users can join channels (JOIN command), leave or part channels, and send/receive messages (PRIVMSG, NOTICE). The UI will display each channel in its own view (tab or split panel) with a backlog of messages. Support channel modes (MODE command) and user modes, with UI indicators for common modes (e.g. show ops (@), voiced (+) users). Users can set topics (TOPIC) and see channel lists.
   * **Nicknames and Presence**: Provide controls to change nick (/NICK command), with automatic handling of nickname in use (errors 433) by appending a suffix or preconfigured alternatives. Support WHOIS and WHOWAS queries with a formatted output pane or popup. Update internal lists of channel users on JOIN/PART/QUIT/KICK events, and handle netsplit join/part storms gracefully (maybe compress or delay updates to avoid UI spam).
   * **Server Notices and MOTD**: Display the Message of the Day on connect, and any NOTICEs or server wall messages distinctly (e.g., in a server tab or console). Errors from server (like banned from channel, etc.) should be shown to the user in context.
   * **CTCP and Other Commands**: Implement client-to-client protocol messages common in IRC clients: responding to CTCP VERSION, PING, TIME requests with configurable replies; allowing the user to send CTCP actions (/me for ACTION) which will be displayed as italicized “\* Nick does ...” messages. Also support sending raw commands (an input box that begins with "/" passes text to server after some parsing).
   * **Encoding**: Use UTF-8 internally for all text. Provide options to convert to other character encodings if needed per network (legacy support), but since modern IRC is predominantly UTF-8, this will be default. Ensure Unicode characters (emojis, non-Latin scripts) display properly.

2. **IRCv3 Extended Support**: To be fully modern, implement the widely-used IRCv3 extensions.

   * **Capability Negotiation**: During connection (CAP LS), negotiate features like `server-time`, `message-tags`, `multi-prefix`, `userhost-in-names`, `away-notify`, `account-notify`, etc., enabling them when the server supports them. The client should automatically request capabilities that it understands. For example, **server-time** allows receiving timestamps on messages (for playback or bouncer use), **multi-prefix** ensures all user prefixes (@+%) are included in NAMES, etc. We will include support for the **monitor** list (to track online status) if available, and other goodies like **invite-notify** so ops see when invites happen.
   * **Message Tags**: Parse IRCv3 message tags on incoming messages (e.g. time stamps, `+draft/reply` tags, etc.) and utilize them. For instance, display the server-time tag as the timestamp of the message (overriding local time if present). If an `account-tag` is present (indicating sender’s services account), we can use that for features like mention highlights or who is identified.
   * **Authentication (SASL)**: Support authenticating with SASL mechanisms during connection (`CAP SASL` and `AUTHENTICATE`). Initially implement common mechanisms:

     * **PLAIN**: Send base64-encoded username/password (likely an email and password or NickServ password).
     * **SCRAM-SHA-256**: For networks that allow secure challenge-response (e.g. Freenode/Libera support SCRAM mechanisms for enhanced security).
     * **EXTERNAL**: Use a client SSL certificate to authenticate (if the user has provided one). The client should allow the user to select a certificate file to present for SASL EXTERNAL.
       If SASL auth fails or is not supported, fall back to traditional NickServ auth (the client can detect NickServ login prompts and auto-send password if configured, as mIRC/HexChat do). We will ensure SASL is integrated with the connection flow so that it happens before channel joins.
   * **Away Notifications**: Implement away-notify so that when other users set themselves away or come back, our user list can reflect that (e.g. dim their nick or show an "away" icon).
   * **Extended Join**: Handle the extended join messages (which include account name and realname in JOIN) if enabled. Show whois info accordingly.
   * **Typing Notifications (intent of writing)**: If the IRCv3 typing notification (echo-message or draft/typing) becomes a standard, consider implementing (though not many networks support it yet).
   * **Batch**: Support batching of messages (e.g. ZNC playback or history replay sends a batch start and end). The client should not flood the UI until batch end, to avoid overwhelm. For example, if a bouncer replays 1000 lines with a batch tag, the UI could aggregate them smoothly.
   * **Other**: Support **invite-notify**, **account-tag**, **message-ids** (for replying or editing if that spec emerges), and **cap-notify** to handle caps changing during session. Essentially, aim for complete IRCv3 Core and as many extensions as practical for a client.

3. **Direct Client-to-Client (DCC) Features**: Many IRC users still use DCC for direct chats and file transfers (especially in certain communities for sharing media). RustIRC will include robust DCC support:

   * **DCC Chat**: Ability to initiate and accept direct chat connections with another user (bypassing the IRC server). The UI can open a special query window for the DCC chat. Use cases include private conversations that are potentially faster or off-record. We’ll implement the handshake (CTCP `DCC CHAT <ip> <port>`) and manage the direct socket connection. Should show the user when a DCC chat is connecting/connected and allow termination.
   * **DCC File Send/Receive**: Users can send files to others via DCC and accept incoming file offers. Implement CTCP `DCC SEND` requests. The file transfer UI will show progress (bytes sent/received, speed, estimated time) and allow pause/cancel if possible. Large file transfers should be supported, ideally with resume.
   * **DCC Resume**: Implement `DCC RESUME` and `DCC ACCEPT` for resuming interrupted file transfers. This is important for reliability. The client will store state of partial transfers and coordinate resume if both sides support it.
   * **Passive DCC/Reverse DCC**: Traditional DCC requires the sender’s client to be reachable (direct IP connection). To handle NAT scenarios, support **Passive DCC** (sometimes called Reverse DCC) where the roles are swapped: the receiving client listens and the sender connects. Some modern clients/networks facilitate this to get around firewalls. RustIRC will attempt a reverse connection if direct fails (possibly an option “reverse DCC” on incoming requests).
   * **XDCC and File Server**: While not a priority for MVP, note that mIRC supports a basic file server where users can request files by pack number (XDCC). We can consider allowing advanced users to enable a simple file server mode or at least easily send multiple files. At minimum, ensure compatibility with XDCC info messages and allow queuing multiple sends.
   * **Security for DCC**: Since DCC exposes IP addresses, the client will warn users appropriately and allow IP masking if possible (though true masking requires a proxy). If feasible, integrate with UPnP to open necessary ports for DCC when behind a router (could be a stretch goal). Also, possibly support encrypting DCC sessions (there have been IRC client addons for encrypted DCC chat). At least, we will allow users to restrict DCC (e.g. auto-ignore DCC from unknown users, prompt on each attempt, configure download directory safely).

   *Rationale:* Despite being an old mechanism, DCC remains the primary means for direct client transfers, allowing bypassing servers for files or private chats. RustIRC including DCC means users won’t need to keep an older client around for those purposes.

4. **SASL and Authentication Mechanisms**: (Related to IRCv3 but called out as its own requirement due to importance.) Provide a user-friendly way to configure authentication for each network:

   * The GUI should have a **Network List** (like HexChat’s) where each network entry can store NickServ login credentials or SASL mechanisms. The user can choose SASL mechanism or opt for nickserv ghosting. This ties into the connection sequence above.
   * For SASL EXTERNAL (cert-based), allow the user to import or specify a client certificate (likely a PEM file path) and use it when connecting to that network’s SSL port. The UI should make it clear when SASL succeeded or failed (maybe output in server tab).
   * If a network doesn’t support SASL or if the user chooses, implement the fallback auto-login: after connecting, and after identifying the “NickServ” service, automatically send “IDENTIFY password” (or whatever the service requires – could template per network).
   * Support **auth with services** beyond NickServ if needed (e.g. some networks have *AUTH* command). We’ll primarily focus on standard NickServ though.
   * Store credentials securely: We will not store plain-text passwords in config if possible. Options: use system keychain on each OS (could be complex), or at least allow omission so that user is prompted at connect. Possibly provide an encryption for config file (user-provided master password).
   * Also support **server passwords** (the PASS command) if the network uses a direct server password (some private servers do).
   * Usability: Indicate in UI (maybe via an icon or text in status) if the user is authenticated (identified) with the network’s services. For example, on networks that support account-notify, we can know when our account is identified.

5. **User Interface (GUI and TUI)**:

   * **Graphical UI**: The primary interface will be graphical, using a modern toolkit. We plan to use a Rust-native GUI library like **Iced** for a polished cross-platform look, or fall back to GTK (via gtk-rs) if needed for better maturity on Linux. The GUI will present a server list and channel list in a sidebar (like HexChat’s tree view option), or alternatively, a tab bar for each open channel (like mIRC and default HexChat). Users can switch between channels by clicking tabs or using keyboard shortcuts (e.g. Ctrl+Tab or Alt+1..9). The message view will show messages with timestamps, nickname coloring, and relevant highlights (mentions highlighted). An input box at the bottom allows typing commands or messages. Key UI elements:

     * **Channel Tree/Tab**: A panel that lists networks and channels. Networks can be expandable to show joined channels and open queries under them. Channels in the list could show an unread message count or highlight indicator if one occurs while not focused.
     * **Nicklist**: A sidebar on the channel view showing the users in the channel, with symbols for op/voice/etc. This is common in GUI clients and helps users see who’s present. It can be toggleable.
     * **Menus/Toolbars**: Basic menus for actions like File (connect/disconnect, exit), Settings, Help, etc. A toolbar could have common actions (connect, join channel, etc.), but we might keep it minimal to reduce clutter.
     * **Customization**: The interface will support themes (colors, font sizes) and possibly different layout modes (e.g., toggle showing nicklist, or using a horizontal vs. vertical split for channel vs. nicklist).
     * **Notifications**: Integrate with OS notifications for events like your nickname being mentioned or private messages received. Make this configurable (on/off, and what events to notify).
     * **Dialogs**: Provide dialogs for common tasks: adding a new network (with predefined choices for major networks), channel list (getting list of channels via LIST command and filtering), DCC send (“Choose file to send to X”), etc., to make these tasks user-friendly rather than requiring command-line usage.
     * **Status Bar**: Show status info like connection lag, current nick, maybe memory usage, or small icons for connection state.
     * **Internationalization**: Ensure the UI text can be translated. Use a crate like `fluent` for multilingual support. The goal is to have RustIRC available in multiple languages (like HexChat and WeeChat have been translated).
     * **Accessibility**: The GUI should be navigable with keyboard (for users who cannot use a mouse easily) and screen-reader friendly (proper labels on UI elements).

   * **Terminal UI (Optional)**: For advanced usage or running RustIRC on a remote server (ssh) or as a background service, we will provide a TUI mode. Using a crate like `ratatui` (Rust adaptation of TUIs) or `crossterm`, we can implement a curses-like interface similar to WeeChat or irssi. This TUI mode would share the core backend with the GUI (so all networking logic is common; just two frontends). The TUI might not have all the graphical niceties, but it should allow core functionality: showing multiple channels (possibly via splits or multiple terminal windows), colorized text, etc. Essentially, one could run `rustirc --tui` to get a terminal client experience. This satisfies power users who prefer CLI and also enables usage on minimal systems.

     * The TUI will support key commands (like Alt+number to switch channel, or using arrow keys to scroll). It can mimic WeeChat’s default keybindings to feel familiar (e.g., Alt+→ to move to next buffer, etc.).
     * We’ll include basic mouse support in the terminal if possible (ncurses can support mouse clicks for selecting windows).
     * While the TUI is not the primary interface for average users, it’s important for RustIRC to not alienate users of WeeChat/irssi. By offering both GUI and TUI, we truly bridge the gap between those worlds.

   * **Window Splitting and Multi-view**: A standout feature to implement (and an improvement over HexChat) is the ability to view multiple channels or conversations at once. For example, a user might want to monitor two channels side-by-side. RustIRC’s UI will allow opening a new window view or splitting the current window into panes (vertically or horizontally). WeeChat and some GUI clients (like Konversation, or mIRC’s MDI windows) support this, and it’s very useful on large screens. The plan is to let advanced users drag a channel tab out to create a separate window, or use a “Split view” command to tile channels. Each pane still shows the full UI of a channel. We’ll have to manage syncing input focus, etc., but the benefit is improved situational awareness (no need to constantly switch tabs during active multi-channel discussions).

   * **Ease of Use Features**: Include conveniences such as:

     * **Input History**: Up/Down arrow in the input box to recall previous messages or commands (per window).
     * **Auto-completion**: Hitting Tab to auto-complete nicknames (context-aware to the channel’s nicklist) or common commands. Possibly also auto-complete channel names after a `#`.
     * **URL Handling**: Detect URLs in chat and make them clickable (launching the default browser). Perhaps also an option to copy them or highlight them differently. (We will sanitize them to avoid issues.)
     * **Media**: While IRC is text, consider a plugin or opt-in feature for media previews (e.g., if a user posts an image link, the client could preview it inline or in a tooltip). This is not standard in traditional IRC clients but is an area of innovation – it could be added in later phases as a plugin to keep core light.
     * **Mentions/Highlights**: Let users set keywords (especially their name) that will trigger a highlight. Visually, highlighted messages could have a different color background. Combined with notifications, this ensures important messages aren’t missed. HexChat and others do this; we’ll include it.
     * **Logging**: Provide chat logging to files (more details in Additional Features below) and an interface to view logs or search them from within the client.
     * **Resizable and Detachable Components**: The user should be able to resize panels (e.g., widen the nick list or hide it entirely). Possibly detach a channel to its own window (particularly on Windows or X11, for multi-monitor usage).
     * **Theme/Appearance Settings**: At least support switching between a default light and dark theme. Advanced: a theme format (like CSS or a simple config) so community can create themes. This was popular with HexChat (themes available on their site).

   * **Context Menus**: Implement right-click context menus on elements like user names (to perform actions like Whois, Query, Op/Deop, Kick, Ban, etc.), and on channel tabs (to close, reload, etc.). mIRC’s UI was known for these convenient right-click ops, and users miss them in clients that lack it. We will make sure channel operators have easy UI for common tasks (ban/kick via menu instead of typing commands), which lowers the barrier for moderation tasks.

   * **Settings Dialog**: A comprehensive settings window to configure all aspects of the client (without editing text files, though we will also allow text config for power users). This includes network settings, UI customization (fonts, colors), logging preferences, alerts, etc. Good settings UX is needed so new users can tweak the client easily rather than searching for commands.

   * In summary, the GUI will aim to be at least as user-friendly as HexChat’s (which was often recommended for newcomers), and the TUI will satisfy the minimalism and scriptability that WeeChat users expect, all using the same core.

6. **Scripting and Plugin System**: A major feature of RustIRC is extensibility through scripts and plugins, combining ideas from mIRC, HexChat, and WeeChat.

   * **Embedded Scripting Engine**: We will embed a lightweight scripting language into RustIRC for on-the-fly scripting. Our plan is to use **Lua** (via the `rlua` or `mlua` crate) as the primary embedded script language. Lua is chosen for its small footprint, ease of learning (often compared to mIRC script in simplicity), and safe sandboxing. This allows users to write scripts similarly to mIRC’s remotes/aliases or WeeChat’s scripts but without needing an external interpreter. For example, a user could write a Lua script to respond to certain messages (like auto-greet someone who joins a channel, or log specific events).

     * We will expose an API to Lua that includes events (on\_connect, on\_join, on\_privmsg, etc.), and functions to send messages, manipulate UI (to an extent), and perhaps storage for script settings. This parallels mIRC’s on-event structure but using Lua syntax.
     * The scripting engine should support running multiple scripts at once, possibly with some namespace isolation. Provide a way to load/unload scripts at runtime (e.g., a `/script load <file.lua>` command or through the GUI’s scripting panel).
     * Key goal: **mIRC-like aliases and popups** can be implemented via scripting. E.g., define a custom command like `/slap` via a Lua script that sends an action message. Or a popup menu that is populated by a script.
     * Safety: The scripting sandbox will by default prevent dangerous operations (or at least warn), as mIRC’s free-for-all scripting led to abuse. Lua can be sandboxed by not exposing file system or OS libraries unless the user trusts the script. This is a security consideration.
   * **Plugin Architecture**: For more heavy-duty extensions, RustIRC will support binary plugins. Likely, we define a plugin interface (in Rust, possibly using dynamic loading of `.dll`/`.so` files). Advanced users or third-party developers can write RustIRC plugins in Rust (or C via FFI) that get loaded at runtime to add new features. This is analogous to HexChat’s plugin system which allowed Python/Perl scripts as “plugins” and C plugins compiled against its API. Initially, we might keep it simple with scripting, but we want the design to accommodate future plugin loading.

     * Out-of-the-box, we plan to include *adapters* for a couple of languages in addition to Lua: for example, a Python plugin that uses Python’s interpreter to run Python scripts (HexChat and WeeChat both support Python scripting natively). If feasible, include Perl as well. However, embedding multiple language runtimes can bloat the application; we might make these optional or load on demand. Perhaps provide a plugin that, when loaded, allows executing Python scripts. This way, users who don’t need it don’t pay the cost. The core idea is flexibility: some users prefer writing automation in Python (more powerful libraries), others like simple Lua, etc.
     * Support for **WeeChat script relay**: Not directly, but since WeeChat has a relay protocol and many mobile clients use it, we might consider implementing a *relay plugin* for RustIRC in the future, or at least ensure parity such that a mobile client could connect to RustIRC similarly. This is advanced and can be deferred.
   * **Script Manager**: Taking inspiration from WeeChat’s built-in script manager, RustIRC will offer a script/plugin manager interface. This could be a window that lists available scripts/plugins (with descriptions) from an online repository (if one exists for our project, or from git). The user can one-click install common scripts (for example, an alias pack, an away logger, fun scripts, etc.). In early versions, this might be as simple as a documentation listing, but ideally it becomes integrated. This feature encourages community sharing and makes it easy for non-programmers to extend RustIRC by using others’ scripts.
   * **Compatibility with mIRC Scripts**: This is a stretch goal and not a primary requirement, but worth noting: mIRC’s scripting language (mSL) is unique, and some users have large scripts written in it. AdiIRC (another client) actually implemented a partial mSL interpreter for compatibility. If time and resources allow, we could consider an **mIRC script compatibility layer** plugin, so that users could run some of their `.mrc` scripts in RustIRC. This would be a big win for adoption by old mIRC users (they could bring over their favorite scripts). However, implementing mSL parsing/execution is non-trivial, so we will likely deprioritize this unless there’s community interest. Instead, we’ll provide guides to translate common mSL scripts into Lua/Python equivalents.
   * **Event Hooks for Plugins**: Design the core such that it emits events that both scripts and plugins can hook into. This means our internal event dispatcher (for receiving a PRIVMSG, for a user joining, etc.) can notify registered callbacks. Performance needs to be considered (don’t want a slow script to stall the network handling; we might execute scripts async or in a limited thread). Possibly queue events to scripts to be handled sequentially to avoid race conditions.
   * **Default Scripts**: We may bundle a few useful scripts by default (likely disabled unless activated) to showcase functionality: e.g., an *auto-op script* (automatically op certain users on join if you have op), or a simple *logger* (though logging might be built-in already), or fun ones like *slap command*. These serve both as features and as examples for users to write their own.
   * **Testing Scripts**: Provide a way in the UI to quickly run small script snippets (like a script console) for those experimenting. Not crucial, but nice for development.
   * *Summary*: RustIRC’s scripting and plugin system will be a core strength, allowing deep customization akin to mIRC’s legendary scripting but with modern language options and safer defaults. This will set it apart from minimal clients and ensure longevity as users can adapt it to new needs (for instance, if an IRCv3 extension comes out after release, someone could write a plugin to support it if the core hasn’t yet).

7. **Additional Features and Utilities**:

   * **Logging**: The client will support logging of conversations to disk. Users can enable/disable logging per channel or globally. Logs should be stored per network and channel (e.g. `~/rustirc_logs/Network/Channel.txt`), typically in plain text with timestamps. Provide options for log rotation (daily files) or size limits. Ensure sensitive info (like passwords in server messages) aren’t accidentally logged or can be redacted. Possibly offer an export in HTML or other format for nicer reading. Logging is important for many users to keep chat history, and was present in all major clients.
   * **Search in Logs**: A nice-to-have: a log viewer or at least an integrated search to find past conversations. This could be implemented later or via a plugin.
   * **Notifications and Alerts**: As mentioned, integrate with desktop notifications (using system APIs on Windows (toast notifications), macOS (Notification Center), Linux (DBus notify) as appropriate). Allow configuring what triggers a notification: private message, highlight, someone says your name, or on any new message in certain channels. Possibly include a “flash taskbar” on Windows like mIRC did, or bouncing dock icon on Mac for attention. Also consider sound alerts (play a sound file on highlight, etc., configurable).
   * **NickServ/Authentication Assistant**: Besides SASL, provide quality-of-life like automatically responding to NickServ authentication requests (if not using SASL), regaining your nick if it’s ghosted (auto-ghost on connect if your nick is in use and identified to services). These are things experienced users often script, but we can have built-in support toggled in settings.
   * **Themes and Appearance**: Support customizable color themes. At minimum, allow user to change the color scheme of messages (e.g. differentiating nick colors, ping highlights, error messages color, etc.). Perhaps offer a few preset themes (a light, a dark, maybe one resembling mIRC default, one like Zenburn, etc.). If using a GUI toolkit like Iced that supports theming, we leverage that. The theme might also cover font choice and size. HexChat had a UI for editing colors of various message types – we should have something similar.
   * **Encryption (OTR/Off-the-Record)**: While not part of IRC standards, OTR is commonly used for private message encryption between clients. None of mIRC/HexChat/WeeChat had built-in OTR by default (WeeChat uses a script for OTR, HexChat via a plugin). RustIRC can differentiate itself by optionally including OTR support for private chats. We can use an existing Rust OTR implementation or bring a C library and interface via FFI. If OTR is enabled for a conversation, the client should manage the key exchange (OTR v3) and show encrypted messages once established. This is a complex feature so it might be delivered as a plugin initially. But it’s worth listing as a goal since privacy-conscious users would appreciate it. At the very least, ensure that if an OTR plugin is not ready at launch, we design the code to allow adding it. Also consider newer encryption schemes (some people use signal-like protocols over IRC via scripts).
   * **Proxy Support**: Enable connectivity through proxies, both SOCKS (v5, maybe v4) and HTTP proxies (CONNECT method for SSL). Provide settings per network for proxy or a global proxy setting. This is essential for users in restricted environments or who want to route through Tor/I2P. Ensure resolving of hostnames can be done client-side for socks if needed. If Tor usage is anticipated, allow an option to not do any DNS leaks.
   * **BNC/Bouncer Integration**: Many power users use bouncers like ZNC or Bouncer as an always-on proxy. RustIRC should be tested with these and ideally provide special support: e.g., when connected to ZNC, automatically request playback of missed messages (`*playback` module or using server-time to detect gap). Possibly allow multiple users on one bouncer connection (ZNC with multiple clients). We can also mimic some bouncer functionality by letting RustIRC itself run headless (TUI mode on a server, then GUI attaches via something – though implementing our own bouncer protocol might be out of scope). At least ensure the client works well with existing bouncers.
   * **Auto-updates**: As a convenience, especially on Windows and Mac, consider implementing an auto-update mechanism (so users can easily get new releases). Perhaps integrate with a hosted update file or use the OS’s update (Windows could use winget or our installer with updates; macOS might use Sparkle if we had a native integration, or just prompt the user). On Linux, this is usually handled by package managers or Flatpak, so less needed there. We can simply notify the user when a new version is available (checking a GitHub releases feed), which is simpler.
   * **Detailed Options for IRC behavior**: Provide settings for things like reconnect on disconnect (and delay between retries), keeping logs of server MOTDs, ability to disable IPV6 if it causes issues, etc. A parity feature from mIRC: **perform** on connect (execute a set of commands automatically upon connecting to a server, e.g., join channels or identify – though SASL covers identify, users might want to auto-join channels, which we will have in the server settings). Essentially, any behavior that advanced users expect to be tweakable should have an option in settings or at least via a config file.
   * **Performance Options**: If users join very large channels (e.g. 10k users), the UI may lag updating nicklist. We could include an option to cap nicklist updates or require a manual refresh for huge channels. Also memory-related options like how much backlog to keep in memory per channel (some clients allow setting how many lines to keep visible).
   * **Compatibility/Quirks**: Where needed, include toggles or support for IRCd quirks or bots. For instance, support the CTCP `CAP` used by some bots to list their capabilities, handle the atypical formatting of some networks (like if a network uses non-standard whois lines, try to parse gracefully). If EFnet still requires +x user mode for cloaking, make it easy to set. These are minor but contribute to polish.
   * **DCC/File Utilities**: Provide a simple file browser for received files (open the folder where files are saved). Possibly a dialog to manage ongoing DCC transfers (like a combined view).
   * **UI Polish**: Small things like clickable nicknames (e.g., ctrl+click a nick in chat to open a query window), drag and drop file to send (drag file onto a query window to initiate DCC send to that user), etc.
   * **Help and Documentation**: Integrate help links or tooltips. For example, a “Help” menu linking to an online manual or showing basic IRC commands. This lowers the entry barrier for new IRC users who may not know commands like /join or /msg.

All the above functional requirements will be built with a focus on matching or exceeding the capabilities of the reference clients. By the final product, a user should be able to do anything in RustIRC that they could in mIRC, HexChat, or WeeChat, and ideally do it more easily or more securely.

### Non-Functional Requirements

* **Performance**: RustIRC should be lightweight in terms of resource consumption. Target to keep memory usage low (a few tens of MB when idle with moderate channels, not hundreds). CPU usage should be near-zero when idle, and efficiently handle bursts of messages (e.g., if a netsplit rejoins flood hundreds of join/part messages, handle without freezing UI by batching updates). The choice of Rust and asynchronous I/O helps here – no busy-wait loops, everything event-driven. Startup time should be quick (a few seconds at most). We’ll use profilers to ensure no hot loops are eating CPU. Networking should be efficient: use non-blocking sockets with Tokio, and be able to handle high latency networks gracefully (pings etc.). For reference, WeeChat and HexChat are known to run well even on low-power devices; RustIRC should aim for similar or better due to Rust’s low overhead.

* **Scalability**: Handle edge cases like a user joining 50+ channels across 10 networks. The UI design (with perhaps multiple windows or many tabs) should remain navigable. Data structures for user lists should handle thousands of nicks (maybe use a sorted list or tree for quick lookup). The network code should handle multiple connections concurrently (Tokio can manage many tasks). Also, if connected to an IRC bouncer that buffers lots of messages (playback thousands of lines), the client should handle that amount of data without crashing. Logging should also handle large logs in terms of disk (with rotation to avoid unbounded growth).

* **Security**: In addition to communication security (TLS, SASL, etc.), ensure the application itself is secure:

  * Avoid buffer overflow or use-after-free issues (Rust inherently helps with this).
  * When interfacing with C libraries (e.g., for GUI or Lua), be careful to wrap in safe abstractions.
  * Validate all inputs: IRC messages can be maliciously crafted (very long lines, weird characters). Implement length checks (IRC protocol lines should typically be <512 bytes, but handle gracefully if more).
  * Redact sensitive data in logs or UI where appropriate (for instance, if a server PASS is used, ensure it’s not accidentally shown).
  * Provide an option to **mask user host** on connect if using a cloak (some clients auto-set mode +x on networks like OFTC/EFnet if available).
  * Protect scripting interfaces: as mentioned, default to safe mode where scripts can’t do file I/O unless allowed, to prevent a rogue script from damaging a user’s system. Possibly sign official scripts or warn if installing third-party ones.
  * Use secure defaults: e.g., enable certificate verification (and allow user to trust custom cert if they run a private server with self-signed).
  * Privacy: No data should be sent to any server other than the IRC servers user configured (except perhaps checking for updates or fetching script list, but that should be opt-in and transparent). We won’t have telemetry.

* **Usability**: Non-functional but crucial. The app should be easy to install and run. On Windows and Mac, provide installers that include all dependencies (no requiring the user to install GTK separately, etc.). The UI should be responsive (no noticeable lag when switching channels or opening settings). Ensure UI text is readable with good default font sizing and contrast (consider dark mode default perhaps). We aim for a **beginner-friendly** experience: for instance, have some preset popular networks in the network list (Libera.Chat, EFnet, Snoonet, etc. with correct ports and addresses) so newbies can connect with one click. Also, provide simple error messages – e.g., if connection fails, show why (DNS failure vs refused vs TLS error). Strive for polish like remembering window size and position between sessions, etc.

* **Maintainability**: The codebase should be organized into modules (network core, UI, plugin manager, etc.) with clean interfaces, to facilitate contributions and future changes. Use Rust’s package management (Cargo workspaces if splitting into multiple crates, e.g., a core library crate and a UI binary crate). We will write documentation comments and perhaps a high-level overview in the repo. Non-functional but related, we plan to include extensive unit tests for core logic (e.g., IRC message parser, configuration loader) and some integration tests (maybe simulate a small IRC server to test connection flows). Aim for at least 80% test coverage on core modules. The architecture (detailed in the next section) will separate concerns to make it easier to test and update parts (for instance, the UI should call into a client state API rather than deeply coupling to network code).

  * Plan for future: If an official IRCv4 or new protocols come, the modular design should allow adding support without rewriting the whole client.
  * Also, maintainability in terms of updating libraries: keep dependencies updated to avoid security issues. Use tools like `cargo audit` to monitor.

* **Compatibility**:

  * **Operating Systems**: We must run on Linux (at least glibc-based distros; if using a Rust GUI that uses system libs like GTK, ensure those are handled or statically linked where possible), macOS 11+ (Big Sur and later, since we may target ARM and Intel Macs), and Windows 10 or later (taking advantage of modern Windows features; we won’t support Windows 7 officially given 2025 context, which is fine as those users have alternatives). Also consider \*BSD compatibility if using portable libraries – since WeeChat runs on BSDs, it would be nice if RustIRC can at least compile on FreeBSD. Not a primary target, but keep code portable (avoid Win32-specific or Linux-specific behavior without guards).
  * **IRC Servers**: Work with all major IRC daemons (InspIRCd, UnrealIRCd, ircd-seven/Charybdis, Bahamut, etc.) and networks (Libera, EFnet, IRCnet, DALnet, etc.). Test on at least a few different ones. Some use slight variations in replies and capabilities, so robust parsing is needed.
  * **Scripts/Plugins**: Ensure backward compatibility for our own scripting APIs as they evolve – e.g., if we release RustIRC 2.0, aim to not break all 1.x scripts. This means careful design of the script API and perhaps versioning of it.
  * **Standards Compliance**: As stated, comply with RFCs. For instance, handle CR-LF properly in IRC messages, support maximum line lengths, etc. Use feature negotiation (like if a server only supports SASL PLAIN, handle that).
  * **Distribution**: On Linux, follow FHS where applicable for config and cache: likely use `~/.config/rustirc/` for user config, `~/.local/share/rustirc/` for logs or scripts, etc., and not clutter home. On Windows, use `%APPDATA%\RustIRC\` for config. This ensures we fit well in each OS ecosystem.

* **Reliability and Stability**: The client should handle network disruptions gracefully – e.g., if connection drops, automatically try to reconnect after a user-configurable delay. If the program crashes (which we strive to avoid), it should not corrupt the config or logs. Use atomic writes for important files (write to temp then rename) to avoid truncation on power loss. Implement proper error handling on threads and asynchronous tasks; e.g., if one server’s task fails, it shouldn’t bring down the whole application. Also, no memory leaks – long uptime should not degrade performance or exhaust memory (Rust helps, but watch out with C libs and ensure we free plugin resources on unload, etc.).

  * Possibly provide an option for an auto-backup of configuration periodically, so user can recover if something goes wrong (like config gets corrupted due to an OS crash).

* **Accessibility**: As briefly mentioned, ensure the GUI is navigable via keyboard alone (tab order through inputs, etc.). For screen readers, using standard UI controls where possible so that they can announce chat messages. This might be difficult in a custom-drawn text view, but maybe we can use an accessibility tree or at least allow copying text to clipboard for screen readers. This is a niche but important non-functional aspect.

* **Legal/License**: Ensure all dependencies are compatible with an open-source release (we plan GPL-3.0 or similar for RustIRC itself, as many IRC clients are GPL). Check licenses of crates to avoid surprises. If we include icons or artwork, make sure we have rights or they are open-source. If distributing on Apple App Store (unlikely), licensing might matter (maybe not applicable here).

By meeting these non-functional requirements, we will produce a client that is not only rich in features but also robust, pleasant to use, and easy to sustain.

### Tech Stack and Dependencies

Leveraging existing libraries will accelerate development. Below are key dependencies and libraries we plan to use in RustIRC:

* **Async Runtime**: `tokio` – for handling asynchronous TCP connections and timers. Tokio is a well-tested async runtime in Rust suitable for our use (multiple IRC connections, each running an event loop). It provides futures, async I/O, and utilities like timeout handling (for ping timeouts, etc.). We’ll also use `tokio-native-tls` or `tokio-rustls` for TLS support to integrate with Tokio’s event loop seamlessly.

* **IRC Protocol**: We will evaluate using an existing IRC parsing crate, such as `irc` (on crates.io) which might provide client functionality. However, to fully control compliance, we might implement our own lightweight parser for IRC messages. The format is simple enough (split prefix, command, params, trailing). There’s also the `ircc` crate or others; if any supports IRCv3 tags and modern features, we could adopt it and contribute if needed. In either case, parsing and formatting IRC messages is core – we will ensure our implementation can parse tags and handle encoding properly.

* **GUI Library**:

  * Primary choice: `Iced` (cross-platform Rust GUI crate) which uses a functional-reactive style and targets Windows, macOS, Linux (via winit + canvas or widgets) all in one. Iced is attractive for being pure Rust and having a modern look. We need to verify it can handle things like text view with color formatting (for IRC colors or styled text). We might need to implement a custom widget for the chat display (to support different colors per message, etc.). If Iced proves limiting, alternative is using GTK via `gtk-rs` (which would be native on Linux, via X11/Wayland, and can work on Windows via bundled GTK, and mac via XQuartz or bundle – though mac users might find a native Cocoa app nicer). There's also `QT` bindings (via PyQt or C++ bridge) but that complicates build and license.
  * Another interesting choice is `fltk-rs` (Rust bindings to FLTK) which is lightweight and MIT licensed, but its look is a bit dated and might not integrate as well with OS.
  * For now, plan: try Iced for a cohesive cross-platform UI. If UI performance or integration is an issue, pivot to GTK for Linux/Windows and perhaps native toolkit for Mac (Cocoa via `cocoa-rs` or `tao`/`wry` combination as used in Tauri).
  * **Terminal UI**: Use `ratatui` (formerly tui-rs) for building a TUI interface. It provides primitives for text layout, lists, etc. We’ll create a simplified version of the UI there.
  * **Text Rendering**: If needed, crates like `unicode-segmentation` and `unicode-width` will help ensure we handle wide characters and combining characters correctly in the UI.

* **Network**: Rust’s standard library covers TCP/UDP. For proxy support, we might use `socks` crate for SOCKS5 client if not writing ourselves. For DNS resolution, use Tokio’s default or `trust-dns` if needed to do async DNS (especially for Tor/proxy, might need special handling).

* **TLS**: `rustls` via `tokio-rustls` for pure Rust TLS (which avoids platform dependency issues). Rustls is secure and fast. Alternatively, for system certificates usage, we can use `native-tls` which wraps OS libraries (Schannel on Windows, SecureTransport on Mac, OpenSSL on Linux). We might prefer rustls for consistency and control, but ensure to include major CA certs (likely via the `webpki-roots` crate) or allow OS root store usage.

* **Config and Persistence**: Use `serde` + `toml` or `serde_json` for config files. TOML is human-editable and good for config (similar to how weechat uses plain text configs). We’ll define config structs and derive Deserialize/Serialize. Possibly use `confy` crate for convenience in managing config paths. For more complex state (like UI layout), we can store small bits in config too.

* **Logging (internal)**: Use Rust’s `log` facade with an env\_logger or similar for debug output. Not to be confused with IRC logging, this is for our application debugging. In release builds maybe minimal logging or user-selectable debug mode.

* **File I/O**: `tokio::fs` for async file operations (like writing logs), or if using sync file writes, ensure to offload to a thread pool (Tokio’s blocking calls or spawn\_blocking) so as not to block the async reactor.

* **Crates for Utility**:

  * `chrono` or `time` for timestamps on logs and messages.
  * `dirs` or `directories` crate to find config/cache directories in a cross-platform way.
  * `notify` crate for watching config file changes or scripts directory (not essential, but could live-reload scripts if changed).
  * `clap` or another argument parser for command-line options (like `--tui` flag, `--config path` etc.).
  * For auto-update, perhaps `self_update` crate if we implement that (handles downloading and replacing binary).
  * If implementing OTR or encryption, `rust-otr` or `libsignal-protocol-c` via FFI could be options.

* **Testing/CI**: `cargo test` for unit tests, possibly `tokio-test` for async tests. Use `mockito` or custom test servers to simulate an IRC server (e.g. have a minimal server that sends known patterns, to test our parsing and state handling).

Dependencies will be chosen to minimize bloat but also to avoid reinventing too much. Each addition will be weighed for maintenance. Given the plan timeline, we stick to well-established crates to reduce risk. The exact versions will be locked in Cargo.toml for reproducibility.

*(Note: All crate uses will comply with licenses and will be credited in our documentation. RustIRC itself will be open-source (GPL or MIT/Apache dual) to encourage contributions.)*

## Architecture Overview

RustIRC will follow a modular, event-driven architecture separating concerns of network communication, core IRC logic, and user interfaces. The design is influenced by the Model-View-Controller (MVC) or Model-View-ViewModel patterns, adapted for an asynchronous environment.

**Layers and Components:**

* **Network Layer**: Responsible for managing connections to IRC servers. This layer handles low-level socket I/O, sending pings, and reading incoming data. It parses raw IRC messages into an internal representation. Each server connection can be an independent async task (Tokio task) that feeds events to the core. The network layer also implements reconnection logic, and keeps track of connection state (handshake done, caps negotiated, etc.). It will likely expose an API like “send command to server” for the rest of the app to use, abstracting away the socket details. If we support multiple connections, there will be multiple instances of this (one per server in a map).

* **Core IRC Logic (State Management)**: This is the brain that maintains the current state of each server and channel. It keeps data structures for:

  * Networks -> Channels -> Users. For example, a map of network name to a struct that contains a map of channels, each containing a list of users and their modes.
  * It processes events from the network layer (like JOIN, PART, PRIVMSG) and updates state accordingly (e.g., add user to channel, append message to buffer).
  * It also triggers outgoing actions like if user requests to join a channel, this core layer updates state (mark channel as joined or pending) and instructs network layer to send the JOIN command.
  * Think of it as the Controller/Model in MVC combined: it knows how to handle each IRC command semantically (update model and maybe call some callbacks).
  * The core also communicates with the Scripting/Plugin subsystem: for each significant event, it will emit an event that scripts can hook (e.g., on\_privmsg, on\_user\_join).
  * Additionally, core handles timing events like sending periodic PINGs or auto-reconnect. Possibly maintain a task for each server for heartbeat.
  * It should be mostly UI-agnostic, but the UI will query it for data (like list of channels to display, etc.). This suggests we might implement an observer pattern or a message bus: when state changes, send a notification to UI layer to update. We could use something like `tokio::sync::broadcast` channel or simply call UI update functions if threads are shared (depending on how we structure threading).

* **UI Layer**: Two main subcomponents: GUI and TUI. These will likely be two different frontends that both interact with the core logic via some interface.

  * The GUI (Iced or GTK) will run in the main thread (for GUI frameworks usually). It might use channels to communicate with core running in background threads. For example, when a user clicks “Join Channel”, the GUI sends a message to core to join, and core replies with success/failure and updates state which triggers GUI to show the new channel.
  * The TUI will run in the terminal, likely also interacting with the same core. Perhaps we can only run one UI at a time (if `--tui` is invoked, no GUI).
  * The UI’s job is presentation and input handling: formatting messages (e.g., applying color codes to actual colored text), arranging windows, etc. It should not contain IRC logic beyond small conveniences (like maybe highlight a message if it contains your nick – though even that could be a core feature).
  * We can implement an **MVC** where the Core is the model, and the UI is the view/controller for user events. With async, we may decide to use message passing: e.g., use an internal “Event Queue” for UI events and core events to decouple them.

* **Plugin/Scripting Layer**: This sits somewhat parallel to UI. Scripts and plugins can register with the core to receive events and perform actions. For instance, a Lua script that wants to do something on message will register a hook; when core processes a PRIVMSG, it will call the script hook (in Lua context) via our embedded interpreter. Similarly, plugins might register command handlers (like a plugin might add a new command /foobar – the core sees an input starting with /foobar and can defer to plugin).

  * This layer will need a manager that loads/unloads scripts (probably part of core or closely tied). It might have separate threads for running scripts to avoid blocking the main loop – or use async for scripts as well (like spawn an async task for a script if it does I/O).
  * Also, if we do multiple languages, each will have its runtime (Python’s GIL, etc., careful integration needed).
  * The plugin layer should treat the core as authoritative for state (scripts can query the core for current users in channel, etc., rather than scripts having their own separate state ideally). So we may expose read-only references or query functions to scripts.

* **Storage Layer**: For config and logs, a simple module to handle reading/writing config files (possibly at startup and when options are changed) and writing logs. The core logic will call into this when needed (e.g., log a message -> call log module asynchronously). This could also manage things like an IRC server list database (list of known IRC networks). This is relatively straightforward.

**Concurrency and Threading**:
RustIRC will heavily use asynchronous programming. The network IO will use async tasks in a Tokio runtime. The UI, if using Iced, can also be integrated with async updates (Iced has mechanisms for futures). We might run the core and network on background threads and have the GUI on the main thread, communicating via channels. For example:

* One approach: **Single-process, multi-thread** – spawn Tokio runtime on a separate thread (or a few threads) for network and core, while GUI runs on main thread. Use thread-safe channels (from `tokio::sync::mpsc` or `crossbeam_channel`) to send events from core to GUI (like “new message in channel X”) and from GUI to core (“user sent message X in channel Y”). This keeps things responsive and separates concerns.
* For TUI mode, we might run everything in one thread or also separate – TUI can block on user input, but we can integrate by using async for input as well.
* We’ll ensure that potentially blocking operations (DNS resolution, file writes) are either async or offloaded, so that our main loop isn’t stuck.

**Data Flow**: A simplified flow of events:

1. A line comes in from the network socket (e.g., `:@nick!user@host PRIVMSG #channel :Hello`). The Network layer parses this into a structured event (perhaps an enum like `ServerMsg::PrivMsg { from: UserId, to: ChannelId, text: "Hello" }` with metadata).
2. The Core receives this event (through an async channel or direct call if in same task) and updates its state (e.g., appends “nick: Hello” to the message list of `#channel` in state). It then generates higher-level events: it might send a UI event like `NewMessage(channel, message_struct)` to notify the UI, and also trigger any scripting hooks (on\_privmsg).
3. The UI layer, upon receiving the NewMessage event, formats it (adds timestamp, color for nick) and displays in the appropriate channel view. If that channel is not currently visible, it could set an unread flag.
4. If a script hook was triggered, say a Lua script that echoes all messages to console, that script’s code runs (maybe on the same thread if quick, or separate).
5. If the message contained a highlight (the user’s nick), the core or UI can detect this and generate a notification event which the UI then uses to show a desktop notification.
6. Suppose the user then types a reply and hits Enter. The UI captures the input from the text box, packages it (e.g., knows it’s to #channel on Network1) and sends an event to core: `UserInput::SendMsg(network_id, target="#channel", text="Hi Nick")`.
7. Core receives that, constructs the raw IRC command (`PRIVMSG #channel :Hi Nick\r\n`) and tells Network layer to send it. It also could optimistically update its state (show the message in the UI even before server echoes it, perhaps using echo-message capability if enabled, or just do it).
8. Network sends it out. If echo-message is enabled, server will bounce the message back which we’d handle like any incoming message but possibly mark it as self-message. If echo not enabled, core itself can insert the message into UI with a “sent” marker to simulate echo.
9. The cycle continues.

This shows a decoupling: UI doesn’t send directly to socket, and network doesn’t manipulate UI, everything goes through core as intermediary. This ensures scripts also see everything (since core can hook both incoming and outgoing messages events for scripts, e.g., for logging).

**Error Handling**: If an error occurs in network (connection lost), the network layer would inform core (perhaps an event “Disconnected(reason)”). Core updates state (mark server offline) and triggers UI update (maybe show a red status or try reconnect after X seconds). The UI might pop up a message “Disconnected: connection reset by peer” in the server tab.

**Diagram:**

Below is a high-level conceptual diagram of the architecture:

```
            +---------------------- RustIRC Core ----------------------+
            |                                                          |
            |  +---------+    +----------------+    +----------------+ |
User Input ->  |  UI/TUI  | ->|    Controller   | -> |   Network I/O  | -> IRC Server
(events)   |  | (View)   |    | (State & Logic)|    |  (Sockets etc)| |
            |  +---------+    +----------------+    +----------------+ |
            |         ^              |    ^               |            |
            |         |              |    |               |            |
            |         |       Events |    | Outgoing IRC  |            |
            |         |     (IRC msgs, UI|    messages    |            |
            |   Render/Update         | commands, etc)   |            |
            |         |              |    |               |            |
            |  +----------------+    |    +----------------+           |
            |  | Scripting/     |<---'    | Plugin API    |           |
            |  | Plugins        |  Hooks  +----------------+           |
            |  +----------------+ (callbacks)                           |
            |                                                          |
            +----------------------------------------------------------+
```

In text form:

* The **UI** receives user input (keyboard, mouse) and produces events (like “send this message” or “join channel X”) that go to the Controller.
* The **Controller** (core logic) processes user events and server events. It updates the internal state (the model) and emits updates to both the UI (to re-render) and to the network (to send commands).
* The **Network I/O** communicates with the server, sending commands and receiving raw messages, which it parses and sends back up to the controller as events (like “message received”).
* The **Scripting/Plugin** system hooks into the controller’s event flow: the controller can broadcast events to scripts, and scripts can call controller functions (through a safe API) to, say, send messages or alter state.
* Arrows indicate flow of data: user input flows into core, core sends commands to network, network sends messages back, core updates UI.
* The UI also reads the current state when rendering (for example, to display the user list or channel topic).

This modularization allows us to work on the network engine, UI, and scripting somewhat independently and makes testing easier (we can test the controller with a dummy network input, etc.).

## Phased Implementation Plan

To manage development, we will break the work into phases with clear milestones. Each phase represents a set of features to design, implement, and test before moving to the next. This phased approach allows incremental progress and the ability to gather feedback early (especially for UI/UX decisions).

### Phase 1: Research, Design, and Project Setup (Weeks 1–2)

**Objectives**: Establish a solid foundation for development by researching requirements in detail, finalizing technology choices, and setting up the development environment and project structure.

* **Tasks**:

  * *Deep Dive into Existing Clients*: Use this time to gather any additional detailed info on mIRC/HexChat/WeeChat. e.g., read HexChat’s plugin API docs, WeeChat’s user guide for any insights. Compile a list of “must-have” features from each. Identify user complaints (from forums/Reddit) to be sure we address them (we have done much of this above). This ensures no major feature is forgotten.
  * *Specifications Finalization*: Turn the requirements in this document into a more granular list of user stories or issues. For example, create a tracker item for “Implement basic IRC connection and message send/receive” etc. Prioritize these.
  * *Select Libraries*: Decide on GUI library by doing quick prototypes. For instance, write a tiny Iced app that opens a window with a text input and see if we can draw colored text. If iced seems to work, proceed with it. If not, evaluate GTK. This phase should end with a decision to avoid later change. Similarly, confirm that `tokio` and `rustls` will be used (likely yes), and test integration (maybe connect to an IRC server and print messages as proof of concept).
  * *Project Setup*: Initialize a GitHub repository (if not already) for RustIRC. Set up the Cargo workspace. Perhaps have separate crates for:

    * `rustirc-core` (handling core logic and network)
    * `rustirc-ui` (for GUI/TUI)
    * or all in one crate to start and refactor later. In this phase, skeleton code can be written: e.g., define an `Event` enum for core events, a basic main that parses args and either launches GUI or TUI stub.
  * *Coding Guidelines*: Decide on coding style (Rustfmt defaults, Clippy lint rules) and set those up. This ensures consistent code.
  * *Documentation*: Start writing a README or wiki for the project with the vision and how to build. Possibly include this planning document for reference.
  * *Risk Assessment*: Re-evaluate any high risks. For example, if we worry about embedding Python and that might be too complex, note that and possibly defer it.
  * *Schedule and Milestones*: Flesh out the timeline for each phase (roughly as we do here) and identify which features can be demoed early. Possibly plan an alpha release after Phase 3 (basic chat working) to get feedback from a few testers.

* **Deliverables**:

  * A Git repository with initial project scaffolding (Cargo files, maybe some dummy code that prints “Hello IRC”).
  * Updated design docs if needed (could be as simple as refining this document with any changes).
  * An outline of the user interface (maybe hand-drawn or wireframe) to guide Phase 3.
  * A list of initial GitHub issues or tasks for upcoming phases.

* **Milestone Criteria**: By end of Phase 1, we should be confident in the tech stack and have no unknown blockers. The repository builds and perhaps connects to an IRC server minimally (even if it just prints raw lines). The team/stakeholders sign off on the design and scope as documented.

* **Risks**: Over-analysis could delay coding – mitigate by timeboxing research tasks. Another risk is underestimating complexity of certain features (like multi-platform GUI); mitigate by prototyping early (spike solutions). If a chosen library seems problematic in prototype, be ready to switch early in Phase 1.

*(This phase is largely about planning, but given we have done a lot of conceptual design already, it should go quickly. We will proceed to implementation in Phase 2 armed with clear goals.)*

### Phase 2: Core IRC Engine Development (Weeks 3–6)

**Objectives**: Implement the fundamental IRC client functionality without a complex UI. This involves establishing connections, sending/receiving messages, and managing basic state. Essentially, by end of Phase 2 we want a working CLI-based client (even if rough) that can connect to a server, join a channel, and echo messages.

* **Tasks**:

  * *Connection Management*: Write code to connect to an IRC server given host, port, SSL option. Utilize Tokio for async. On connect, send the NICK and USER commands properly. Handle the initial registration flow (receive server welcome, join channels if autojoin configured).
  * *IRC Message Parser*: Implement a parser for raw IRC protocol lines. This includes splitting prefix, command, params, trailing. Consider using an existing crate or write from scratch with careful handling of the RFC grammar. Include support for tags (start with `@`).
  * *Data Models*: Create structures for Network (connection info, lists of Channel objects, etc.), Channel (name, topic, list of users), User (nick, maybe user\@host and metadata). Manage these in a central state (like a struct `ClientState` that contains a list of networks). Implement methods to look up and update these (e.g., `state.add_user(network, channel, nick)` for JOIN).
  * *Basic Commands*: Support at least the following user commands in this phase:

    * JOIN, PART (join channel, leave channel).
    * PRIVMSG, NOTICE (send messages).
    * NICK (change nickname).
    * QUIT (disconnect).
    * Perhaps WHOIS (to test a multi-line response parsing).
      Essentially, enough to simulate a normal chat session. This also means handling the corresponding incoming events (e.g., on JOIN from others, update state; on KICK, remove user; on MODE changes, update user/channel modes; on PING, reply with PONG automatically in network layer).
  * *SASL (Plain)*: Implement the simplest SASL mechanism if credentials are provided. For now, maybe just SASL PLAIN since it’s straightforward (send base64 user/pass). This could be tested on networks like Libera.Chat. Or skip SASL in Phase 2 if time is short and do in Phase 5, but implementing it early ensures the architecture can handle auth steps.
  * *State Updates & Callbacks*: When a message is received, update the data model (like append message to a channel’s buffer). For Phase 2, since UI is minimal, we might just log these to console for debugging. But design the callbacks for when we integrate UI later.
  * *Temporary CLI*: Build a simple text interface to test the core. For example, read from stdin and if user types "/join #channel", call the core join function; if they type text, send to current channel. This is just for development verification before GUI is ready. It could be as simple as one channel at a time view in terminal.
  * *Testing*: Write unit tests for parser (feed known IRC lines and assert correct struct output). Simulate partial messages (split across TCP frames) to ensure our buffering logic in network layer works (Tokio’s codec can help here possibly). If possible, create a test that uses an open IRC server or a local dummy server: e.g., connect to irc.libera.chat on a test nick and join a empty channel, then send a message, see if we get it back (with echo-message or via a second client). Alternatively, run a local instance of a simple IRC server (like `ngircd` or `oragono` in test mode) for integration tests.

* **Deliverables**:

  * **RustIRC Core v0.1**: A console-based application (or a set of library functions) that can do basic IRC. This might be presented as “you can connect to IRC and chat using a rudimentary interface”.
  * Code for network handling and message parsing, with documentation comments.
  * A few example config files (maybe a TOML that lists a test server to connect to) to demonstrate usage.
  * Test results indicating that core functionalities (connect, join, message, ping/pong) work reliably.

* **Milestone Criteria**: By end of Phase 2, we should be able to connect to an IRC network (e.g., Libera.Chat), join a channel, and see messages we type appear (if we test with two instances or using another known client). The program should handle multiple channels per server, and multiple servers (maybe via separate instances or ideally within one run if already implemented). Basically, the IRC engine is proven functional. If we run the client in verbose mode, it should show incoming and outgoing IRC lines appropriately, confirming our parser and serializer.

* **Example**: A snippet to illustrate Phase 2 output (in a test scenario):

```rust
// Pseudo-code for connecting and sending a message in core
async fn connect_and_send() -> Result<()> {
    let server = "irc.libera.chat";
    let port = 6667;
    let mut stream = TcpStream::connect((server, port)).await?;
    let nick = "RustIRCTest";
    let user = "rustirc 0 * :Rust IRC User";
    // Register
    stream.write_all(format!("NICK {}\r\nUSER {}\r\n", nick, user).as_bytes()).await?;
    // Join a channel
    stream.write_all(b"JOIN #rustirc-test\r\n").await?;
    // Read one message as demonstration
    let mut buf = [0u8; 512];
    let n = stream.read(&mut buf).await?;
    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
    // Send a message
    stream.write_all(b"PRIVMSG #rustirc-test :Hello from RustIRC core!\r\n").await?;
    Ok(())
}
```

*The above is a simplification. In the actual implementation, we’d integrate with our parsing and event system rather than raw println.* But it shows the basic idea of sending commands and reading responses.

* **Risks**: IRC protocol might have many edge cases (like multiline RPL MOTD, or weird formatting in modes). Risk: spending too long perfecting parser. Mitigation: implement common parts and add edge cases as encountered. Another risk is handling concurrency issues (like two messages arriving at once). Tokio’s single-threaded executor by default should sequence them, so it’s fine, but if using multi-thread runtime, ensure state is protected (use Mutex or run all on one thread to simplify). We will likely run one core task per connection, each on possibly the same runtime thread, which should be fine.

By completing Phase 2, we have the heart of the client ready and can move onto building a user-friendly interface around it.

### Phase 3: Graphical User Interface (GUI) Implementation (Weeks 7–10)

**Objectives**: Create the GUI for RustIRC and integrate it with the core from Phase 2. By the end of this phase, RustIRC should have a working desktop application interface where a user can perform basic IRC operations (connect to server, join channel, send/receive messages) through windows, menus, etc., instead of the console. The focus is on Linux and Windows initially, with macOS adjustments if needed.

* **Tasks**:

  * *Framework Setup*: Initiate the GUI using the selected library (Iced or GTK). If Iced, set up the main application struct, define the message types for Iced (for events), and the basic view layout (maybe a sidebar + main chat area + input box). If GTK, design the UI with either code or a UI file (Glade) and connect signals.
  * *Server/Channel Management UI*: Implement the UI component that shows a list of networks and channels. For example, a tree view in a sidebar. In Iced, this could be a `Column` of servers each with a nested `Column` of channels when expanded. Allow selecting a channel to bring its chat into focus.
  * *Chat Display*: Create a scrollable text area where messages appear. Initially, get simple text display working (e.g., each message on a new line, maybe prefix with nick). Then enhance formatting: parse IRC formatting codes (like color codes ^C, bold, underline, etc.) and apply styles. If using Iced, we might need a custom widget to render formatted text; if using GTK, perhaps a `GtkTextView` with markup (pango markup can handle colors and bold).

    * Also include timestamp on each message (maybe configurable 12h/24h).
    * Possibly differentiate own messages vs others (color or alignment).
  * *Input Box*: Add a text input field for the user to type messages or commands. Wire it so that when Enter is pressed, we capture the text. If it starts with '/', treat it as command (remove the slash and interpret or pass to core’s command handler). If not, it’s a normal message to current channel. After sending, clear the input.

    * Implement input history: store last N messages/commands; support up/down to navigate through them when focus is in input.
    * Support tab-completion for nicknames: detect when Tab is pressed in the input; if the text before cursor matches a prefix of some nick in channel, complete it. This can be done by querying the core’s user list for that channel.
  * *Menus and Actions*: Add a menu bar or context menus:

    * A “File” or “RustIRC” menu with items like Connect, Disconnect, Quit.
    * A “Server” menu for adding/removing servers (or maybe a toolbar button that opens Network List dialog).
    * A “Help” menu linking to documentation.
    * Right-click context menu in chat view or nick list: for chat view right-click on a nick -> options like “Whois”, “Query”, “Op/Deop/Kick” if you have privileges. This requires knowing user’s status, which we have in state.
    * These menu actions when triggered should call core functions (e.g., if user chooses Kick on someone, call core to send a KICK command).
  * *Dialog: Network List*: Implement a dialog (window) that lists configured networks and allows editing/adding. Similar to HexChat's network list. For now, storing networks in config might be enough, but a UI to avoid editing config by hand is good. Fields: address, ports, SSL yes/no, nickname, username, realname, auto-join channels, SASL or NickServ password. This is a chunk of UI work but improves usability greatly. The dialog can populate from a default list (some popular networks pre-filled).

    * Also allow quickly connecting/disconnecting through this UI (e.g., double-click network to connect).
  * *State Integration*: Connect the GUI to the core state. Possibly run the core in a background thread and communicate via channels. For example, when a new message arrives in core, send a message to the GUI thread to add a line. In Iced, this might be done by generating an Iced Message (maybe via a subscription or by waking the UI with new data). Need to ensure thread-safe transfer of data (use thread-safe data structures or copy small bits).

    * We might decide to run the core loop in the main thread as well if easier, but that risks blocking UI, so probably keep them separate.
    * For Phase 3, if integration is complex, we might for simplicity integrate core in the same thread and use async tasks with something like iced\_futures to poll. But have to be careful not to block UI.
  * *Testing UI*: Manually test by running the app: use the network list to add a test server (maybe irc.libera.chat), connect, join channel via UI (maybe an input like “/join #test” or a join dialog). Then verify sending and receiving messages works and they display properly. Fix any issues in parsing or state management that UI testing reveals (for instance, maybe our user list wasn’t updating on someone joining, now we see it).

    * Also test resizing windows, switching between channels (the content should change accordingly).
    * Test multiple networks: connect to two servers at once, join channels on both, ensure UI segregates them (the tree view should allow selecting channels from either).
  * *Cross-Platform Testing*: At least test on Windows and a common Linux desktop (e.g., Ubuntu with GNOME). On Windows, ensure the program window appears correctly, fonts are fine, etc. There might be Windows-specific tweaks (like using a different font by default since monospace might not look good if not available). If possible, test on macOS (even if we haven’t done any Cocoa specific coding, Iced should handle it).
  * *Basic TUI (if time within this phase)*: Possibly also implement a very rudimentary TUI mode to ensure core separation works. But this might move to Phase 4. If Phase 3 UI work is heavy, focus on GUI first.

* **Deliverables**:

  * **RustIRC GUI Alpha (v0.3)**: an application that a user can run and perform basic IRC chat in a graphical window. This will still be rough (not all features, minimal polish), but real enough to use in daily IRC for simple chatting.
  * Screenshots of the application connected to a network and in a channel, demonstrating the interface (to share with team/community for feedback).
  * Possibly a short video or gif showing multiple channels and messaging (if needed for feedback).
  * Updated documentation (in README or wiki) on how to use the GUI, what works/doesn’t.
  * If any major workarounds were needed (e.g., if we discovered an Iced limitation and implemented a custom widget), document that in code or comments.

* **Milestone Criteria**: By end of Phase 3, one should be able to use RustIRC for real IRC conversations in at least a basic way. Specifically:

  * You can open the app, configure a server (or use a default), connect to it.
  * You can join a channel (via UI or typing `/join`).
  * When someone speaks, you see the message in the chat window.
  * You can send messages and others see them.
  * No critical crashes during basic operations.
  * The UI is responsive (doesn’t freeze when messages come rapidly).
  * The essential UI components (channel list, chat area, input, nick list) are present.

If this criteria is met, we effectively have an MVP of the client (though lacking scripting and many advanced features).

* **Risks**: GUI development often uncovers performance issues or complexity (like handling thousands of messages in a text view, or long nicklists causing slow layout). We should test with a reasonably active channel to ensure it handles load. If Iced is too slow rendering a big scroll of text, we might need to optimize by limiting what's drawn or use a virtualization technique. Another risk is that multi-threading the core and GUI might introduce synchronization bugs (e.g., UI tries to read state mid-update by core). We mitigate by careful locking or by scheduling updates only via message passing (no direct shared mutable state).

  * Also, cross-platform differences: fonts, DPI scaling (make sure UI scales on high DPI monitors), etc. We might not catch all in Phase 3 but note them for polish in Phase 6.

We’ll consider Phase 3 a success when non-developers could try the client for casual chat and not run into major issues. At this point, we might release an **alpha** build to a few community members for early feedback on UI/UX.

### Phase 4: Scripting and Plugin System (Weeks 11–14)

**Objectives**: Introduce the extensibility features – integrate a scripting engine (Lua) and a plugin interface. Also implement some common customization features (aliases, possibly a basic plugin manager). By end of Phase 4, RustIRC will allow users to run simple scripts (e.g., auto-respond bot, custom commands) and perhaps load compiled plugins, making it competitive with mIRC/HexChat in terms of scriptability.

* **Tasks**:

  * *Lua Integration*: Add the `rlua` or `mlua` crate and initialize a Lua interpreter within RustIRC. Design how the core will expose API to Lua. Likely, we create a Lua context and register some functions/tables:

    * e.g., a global `irc` table in Lua, with functions like `irc.send(channel, msg)` to send a message, `irc.get_users(channel)` to get user list, etc.
    * Also set up callback registration: e.g., Lua script can define a function `on_privmsg(server, channel, nick, msg)` and we arrange to call that from Rust when a PRIVMSG event happens.
    * Consider storing Lua scripts in files (e.g., `scripts/` directory) and at startup (or on demand) load them.
    * Provide a command in the client (like `/script load <name>`) to load a script file, and maybe `/lua` to run an arbitrary Lua snippet for testing.
    * Ensure that a crash in a Lua script (runtime error) doesn’t crash the whole app: use pcall (protected call) to catch errors, and report them to user (maybe print to status window).
    * Memory: set limits if possible to avoid runaway memory usage from scripts.
    * Example: Implement a simple script example such as “auto greeter” that listens for join events and sends a greeting. Use it as a test case.
  * *Plugin Architecture*: Determine how to allow binary plugins. Possibly define a trait for plugins in Rust, and allow dynamic loading via `libloading` crate. For instance, a plugin DLL could expose a `register_plugin` function which we call with our plugin API struct. Given time constraints, focus might be more on scripting (Lua/Python) because writing Rust plugins and dealing with FFI can be complex. However, lay groundwork: define data structures for plugin metadata, and how core can keep track of loaded plugins (script or binary).

    * If feasible, implement one example binary plugin: e.g., a plugin that adds a command `/hello` which prints "Hello world". This would involve writing a small Rust lib crate that uses our plugin API to register the command.
    * Ensure unloading works (might skip actual unload to avoid unsafe issues, maybe just disable plugin functionality).
    * If not fully implementing binary plugins now, at least structure the code so that adding them later is possible without big refactor (e.g., our script hooks and plugin hooks use similar event dispatch).
  * *Alias and Command System*: Users often want to create shortcuts for commands. We can implement an alias feature where a user defines something like `/wc` = `/join #weecoders` (just an example). Perhaps simplest to treat aliases as a mapping in config or a Lua table that Lua can also handle. Alternatively, implement in core: when user inputs a command, check alias map first. This might not require scripting at all, can be just a config thing.

    * Also implement **timers** or **scheduled commands** if easy (like mIRC has /timer). This could be done via scripting or core.
    * Implement **popup menus** customization if time (mIRC allowed editing the right-click menus via scripts). We might postpone to Phase 6 polish if not critical.
  * *WeeChat-like script manager*: Possibly start a simple version. For now, maybe just a command `/script list` that lists scripts in a known online repository (if we create one). Or at least list locally installed scripts. If we have an online repo (maybe a JSON index on our GitHub), we could attempt to fetch it (using `reqwest` crate or so) and then allow `/script install <name>`. This is a bonus, not a core necessity in Phase 4, but including basic support would be nice. If time short, we note it for Phase 5 or 6.
  * *Documentation & Examples*: Write a few example scripts (in Lua) to demonstrate usage: e.g., one that does the classic "slap" command, one that logs all messages to a file (though we have logging, but as example), one that interacts with an external API just to show power (maybe a script that upon command fetches a joke from an API and posts to channel).

    * Possibly also integrate a scripting language like Python: If many users desire Python scripting (since HexChat users might want to reuse Python scripts), try to embed Python too (using `pyo3` crate). This is more complex, so if doing it, do a limited subset (like only allow running scripts manually).
    * It's okay to decide that for 1.0, Lua is the officially supported embedded language (to keep bloat low) and that others can be added via plugins or future updates.
  * *Security Checks*: Review what the scripts can do. If using Lua, by default it has standard libraries including file IO – we might want to remove or restrict that if concerned. Or at least warn users that scripts from untrusted sources can be dangerous (like any IRC script).
  * *UI for Scripting*: Perhaps add a "Scripts" window in settings that lists loaded scripts and allows toggling or unloading them. For now, maybe not fully needed; command-line commands might suffice for tech-savvy users.
  * *Testing*:

    * Write tests for at least the Lua integration: e.g., call a Lua function from Rust and ensure it executes, test that an error in script doesn’t crash core.
    * Manually test by writing a small Lua script to respond to a keyword and seeing that it indeed responds in client during runtime.
    * If adding Python, test that as well (for example, a small Python script printing to console).
    * Test memory: run a script that allocates something and ensure no leaks (maybe use Valgrind or similar if any C involved).
* **Deliverables**:

  * Scripting subsystem integrated into RustIRC, with at least Lua support. Users (and developers) can now extend the client’s behavior.
  * One or two built-in example scripts possibly shipped with the client (to showcase capabilities, and possibly provide useful feature like e.g. an "away logger" that notes who mentioned you while away).
  * Updated user documentation for how to use scripting: i.e., how to write a Lua script for RustIRC, list of available API functions, etc. This could be a section in a README or a separate doc file.
  * If binary plugin support is operational, then an example plugin (source and maybe built .dll for demonstration) and documentation on plugin API.
* **Milestone Criteria**: By end of Phase 4, RustIRC should support:

  * Loading a Lua script file (either via a command or auto-loading from a directory).
  * That script receiving events (like message, join, etc.) and being able to perform actions (send message, etc.).
  * We should see that a user can customize the client significantly without altering Rust code (which is the point of scripting).
  * Also, no stability regression: the client should still run normally with no scripts loaded.
  * If we advertise plugin capability, at least demonstrate a working plugin that extends functionality beyond what a script can do (though in many cases scripts suffice).

A possible demonstration: Write a Lua script that, say, whenever someone says "cookie", your client responds with "nom nom cookie". Load this script and show it working in a test channel (one can simulate someone saying cookie). If that works, it proves event hooking and sending from script works.

* **Risks**: Integrating scripting can introduce instability (a bug in our event calling could crash, or a script could inadvertently cause performance issues like an infinite loop). We mitigate by sandboxing and by maybe running scripts in separate thread or limiting what they can do (though Lua is typically run in the same thread for simplicity). We'll need to carefully test that scripts can't deadlock the UI (e.g., a script doing a long computation might block if run on main thread – maybe offload script execution to a separate task if heavy).

  * Another risk is complexity creeping up: balancing time implementing a full-blown plugin API vs just making sure basic things work. Possibly we focus on Lua as it's simpler, and document that more plugin languages can be added in future.
  * Ensuring that our API covers what users need: We should think of common scripts (like auto voice on join, custom commands) and ensure the API allows those (like need ability to catch join event and send MODE +v; yes).
  * Security risk: A malicious script could do harm (like delete user files if we leave os.remove available). The safe approach might be to remove `io` and `os` libs from Lua environment, or at least mention risk. We can default to safe and allow user to enable full Lua if they want.

Phase 4 brings RustIRC to a level where advanced users can start adopting it, because they can script their workflow. This is a big differentiator and thus important.

### Phase 5: Advanced Features and Protocol Extensions (Weeks 15–18)

**Objectives**: Implement the remaining advanced features that were deferred: full DCC support (file transfers), any remaining IRCv3 capabilities (like multi-prefix, chathistory if possible), better SASL mechanisms, and other polish like the notification system, OTR encryption if possible, etc. Essentially, everything needed for parity with established clients should be done by end of Phase 5.

* **Tasks**:

  * *DCC Chat & File Transfer*: Complete the Direct Client-to-Client features:

    * **DCC Chat**: Create a UI for handling DCC chat requests. Typically, another user will CTCP you: `PRIVMSG you :\x01DCC CHAT chat 192.168.x.x port\x01`. We need to parse that (in core, as CTCP messages) and then prompt the user: "User X wants to start a private DCC chat session. Accept?" possibly with target IP/port info. If accepted, attempt to connect to that IP/port via a new TCP socket. Once connected, open a chat window similar to a query but labeled as "DCC Chat with X" and allow messaging over that socket (which we treat separate from the IRC connection). Implement reading/writing from that socket and display in window. DCC chat messages are just raw text lines (no IRC formatting), but we can show them normally.
    * **DCC Send**: For sending files: Provide a UI way to initiate (maybe a right-click on a user "Send File..." which opens file picker). Or a command like `/dcc send <nick> <file>`. When invoked, get the file size, your external IP (this is tricky behind NAT; maybe use local IP and if using reverse DCC, we might do that variant). Send CTCP offering the file: `DCC SEND filename size ip port`. Then listen on a socket for the incoming connection. Show a progress bar dialog "Sending file to X".
    * **DCC Receive**: When a DCC SEND offer comes in (CTCP), show a dialog: "User X wants to send file Y (Z bytes). Accept/Decline". If accept, connect to them (or if it's reverse DCC, listen and send a notice back, but likely we connect to their ip/port). Save to a specified download directory (ask or use a default, ensure path is safe). Show a progress bar for download, allow pause (DCC resume).
    * **DCC Resume**: Implement logic: if a user cancels or connection drops, allow them to resume. Usually triggered by either user: e.g., if receiver had partial file, they send DCC RESUME <filename> <position> back. Core should handle that handshake: on receiving a RESUME request for a file we offered, reposition file pointer and send ACCEPT, etc. This can be complex but we try to implement at least basic resume if we control both ends (for our client).
    * **Edge Cases**: Large files (32-bit sizes in old protocol vs 64-bit), multiple simultaneous DCCs, NAT issues. Document that if behind NAT, user might need to manually set IP/port or use passive if remote supports. Possibly implement an option "DCC external IP override" in settings for those who know to configure it.
    * Test DCC between two instances of RustIRC or with another known client (e.g., HexChat) to ensure compatibility.
  * *SASL EXTERNAL & SCRAM*: Extend SASL support:

    * **EXTERNAL**: If user has a client cert configured and the network supports EXTERNAL, modify our connect flow to attempt it. Ensure our TLS library can use a client certificate (with rustls, we can configure with a `ClientConfig` including the cert and key). If network says SASL PLAIN failed and we had a cert, try EXTERNAL as fallback if user configured that priority.
    * **SCRAM-SHA-256**: Possibly use a crate like `scram` to implement the challenge response. SASL SCRAM involves multiple back-and-forth messages (server sends nonce, etc.). Implement state machine in core for it. If this is too much effort for now, we might skip if not many networks require it, but ideally include it because it's more secure than PLAIN. We can test on networks like Libera (which supports SCRAM).
    * **More Mechanisms**: Some servers have `ECDSA-NIST256P-CHALLENGE`. That one requires using a private key to sign a challenge (used in NickServ CERT FP). We likely skip it unless there's an easy crate to handle it, because it's rare and EXTERNAL covers certificate auth anyway.
  * *IRCv3 History*: If a network supports the `chathistory` extension or ZNC playback, implement a way to request it. Possibly on join, if enabled, send a CHATHISTORY command to get last N messages if we were offline (like how IRCCloud does backlog, or this might be automatically done by bouncers via `batch`). We might not do a full UI for scrollback in Phase 5, but prepping the core to handle BATCH events and tags like `draft/reply` from history would be good (so that replayed messages show with correct original timestamp and maybe marked as "old").
  * *Notifications Implementation*: Tie in the highlight/mention detection to actual system notifications. Use appropriate crates:

    * On Linux, maybe use `notify-rust` crate to send desktop notifications.
    * On Windows, there's no built-in Rust crate for toast yet (maybe use `windows-rs` to call WinRT notifications, or a simpler approach to flash the window and play a sound).
    * On Mac, use AppleScript or `notify-rust` might also work if it supports Mac (it might via AppleScript).
    * If cross-platform is too messy, possibly rely on a simpler route: e.g., for Mac and Windows, just play a sound and flash taskbar icon if possible (some GUI libs allow setting urgency hint).
    * Provide user settings to turn on/off notifications or set conditions (only notify if away or if window not focused).
    * Test notifications by triggering a mention from another user or using self in two clients.
  * *OTR or Encryption*: If time allows, attempt at least rudimentary Off-the-Record for private messages:

    * Perhaps use an existing implementation. There's a pure Rust crate `snow` but that's for Axolotl (not OTR). There's also some OTR v3 code in C (libotr).
    * Considering effort, it might be too late to implement fully in Phase 5 unless there's a wrapper crate. If not doing now, ensure plugin can provide it (maybe plan an OTR plugin post-1.0). Possibly mark as future enhancement and not block 1.0 if time is short.
    * If implementing: allow user to start OTR with a command or automatically if both clients support. Manage the OTR state machine (AKE, message encryption, SMP for verification).
    * This is expert-level feature; not all users need it, so it's optional.
  * *UI Enhancements*: By now, probably user testing revealed some UI needs:

    * E.g., add the ability to open multiple channel windows (maybe not fully done in Phase 3).
    * Implement window splitting if not done.
    * Possibly implement nick coloring (assign random colors to nicknames consistently).
    * Finish settings UI: allow editing all preferences (maybe implement a preferences dialog with tabs for UI, alerts, etc.).
    * Ensure things like channel mode changes are visible (perhaps in status line or as system messages in channel).
    * Fine-tune default theming (maybe choose a nice dark theme by default given many prefer that, with option to switch).
    * Add more keyboard shortcuts for power usage (like a key to toggle nicklist, F keys for switching networks, etc.).
  * *Cross-Platform Polishing*: Ensure Mac-specific items if needed (like proper menu bar integration – on Mac, app menus appear at top bar, etc. If using Iced, might not handle that, but at least ensure Ctrl vs Cmd keys, etc).

    * Also prepare installers: by now we should think how to package. For Windows, maybe use WiX or Inno Setup to create an installer. For macOS, create a .dmg with the app bundle (maybe using `cargo-bundle`). For Linux, maybe a Flatpak manifest or at least AppImage as a simple approach.
    * These packaging tasks might start in this phase, or at least planning them, since Phase 7 is about release but we can parallelize some packaging earlier.
  * *Testing & Bugfixing*: Phase 5 is a lot about rounding out features, so heavy testing is needed:

    * Test DCC extensively: send a variety of files, test resume by canceling mid-way, test passive DCC by toggling who initiates.
    * Test connecting to many different networks for compatibility: join EFnet (no SASL, might require NickServ manually, see if anything breaks), a UnrealIRCd server (some extended WHO, etc.), test on a bouncer (ZNC) to see if logs replay correctly.
    * Perhaps ask some volunteers to run the client and report issues (if comfortable at this stage).
    * Use memory/debug tools to catch any memory leaks or handle counts (especially with scripts loaded/unloaded).
* **Deliverables**:

  * Fully functional DCC chat and file transfer in the client, with a user-friendly UI (progress bars, accept dialogs).
  * Extended authentication support and verified working login on major networks that require SASL (e.g., Freenode legacy, Libera, etc.).
  * A feature-complete RustIRC **beta release** (v0.9 perhaps) that has all major features enumerated in the requirements. We might call this a feature freeze point.
  * Update documentation: ensure user guide covers new features like "How to send a file", "Setting up SASL with NickServ" etc. Possibly update the website or README with screenshots of these features.
  * If OTR is implemented or integrated via plugin, document how to use it (maybe via a plugin).
* **Milestone Criteria**: After Phase 5, RustIRC should be at parity with or exceeding the capabilities of mIRC, HexChat, and WeeChat in their common use cases. Criteria:

  * A user can do everything in RustIRC they would expect: secure connect (TLS), authenticate (SASL), join channels, chat, set away, change nick, use CTCP (version replies etc.), DCC send/receive files with peers (tested with at least one other client type).
  * The client supports IRCv3 features in practice (tested on an IRCv3 server with things like multi-prefix, who knows – check if multiple modes show, etc. or capabilities like `invite-notify`).
  * The client can be customized via scripts to do non-trivial tasks, confirming that power users are covered.
  * The UI is stable enough to handle daily IRC usage without crashes or major glitches.
  * All high-priority bugs found in testing are resolved.

At this stage, we likely label the build as "Beta" and prepare for public testing. Usually, following this phase, we might have a couple weeks of just bugfixes and user feedback incorporation before calling it 1.0 (that's Phase 6).

* **Risks**: The advanced features, especially DCC and encryption, involve network intricacies which might be tricky. DCC in particular might fail in NAT environments; manage expectations in documentation (like "if behind NAT, either use passive DCC or open ports, etc."). Another risk is time; these could slip if earlier phases took longer. If needed, de-scope things like OTR or some minor IRCv3 features to post-1.0, focusing on the absolutely necessary (DCC and SASL are must, OTR can be later).

  * The UI might become cluttered with new dialogs and features; ensure usability doesn’t suffer (maybe get UI feedback from a few testers, adjust layout if needed).
  * Performance risk: file transfers and GUI updates might need separate threads to not freeze the UI. We'll ensure to perform file I/O in background tasks, only updating progress on the UI thread periodically.

Phase 5 marks code completion of major features. Next, we focus on quality (Phase 6) and release process (Phase 7).

### Phase 6: Testing, Optimization, and Stabilization (Weeks 19–22)

**Objectives**: Rigorously test the software in various scenarios, optimize any performance issues, fix remaining bugs, and polish the user experience. Also, improve code quality (refactoring, adding tests) and ensure documentation is comprehensive. By end of Phase 6, RustIRC should be stable and ready for a 1.0 release candidate.

* **Tasks**:

  * *Comprehensive Testing*:

    * Write a detailed test plan covering: connecting to multiple networks, channel text high volume, error conditions (server disconnects, reconnect logic), scripting edge cases (script throws error, script heavy load), DCC under different conditions, UI resizing and multi-monitor, etc.
    * Execute tests on different platforms. For example, on Windows, test all major features (especially file paths, because path separators differ; ensure config and downloads go to correct directories on Windows, and that Unicode in file names work on Windows).
    * Test on macOS thoroughly if not done: ensure Cmd key shortcuts, retina display scaling is okay, etc.
    * If possible, test on a low-power device (like a Raspberry Pi with Linux) to see performance if applicable.
    * Bug Fixing: Triage any bugs found and fix them promptly. If some are non-critical and time is short, mark for post-1.0, but aim to fix all critical issues (crashes, data loss, major UI issues).
    * Utilize any continuous integration: if we have unit tests/integration tests, run them on CI for Linux, Windows, Mac to catch platform-specific fails (like path issues or locale differences).
    * Security Audit: Go through code or use tools like cargo-audit, ensure no dependency vulnerabilities, and review areas like file permissions (e.g., logs not world-readable if sensitive?), memory handling at FFI boundaries.
    * Possibly do some fuzz testing of the IRC message parser to ensure it doesn’t panic on weird input.
  * *Performance Profiling*:

    * Use a profiler or instrumentation to find any slow parts. For instance, if the UI lags with 10,000 messages, find out if it's the text rendering or maybe inefficient data structure. Optimize accordingly (maybe only keep last X messages in memory for UI, unload older lines except in log).
    * Check memory usage: use something like Valgrind or just track memory over time. Ensure no major leaks (especially from C libraries or if any static caches). If memory usage is high, see if can reduce (e.g., only generate pixmaps for avatars if we had any, etc., but we likely didn't have heavy media).
    * Optimize network: though IRC is low bandwidth, ensure we handle large channel user lists well (maybe optimize how we store and display them).
    * Specifically profile script execution if many hooks, ensure it’s not bogging down (maybe limit frequency of certain events if needed).
  * *User Interface Polish*:

    * Refine any icons or graphics (maybe create an application icon for RustIRC).
    * Ensure consistency (all dialogs have proper titles, focus default buttons, etc.).
    * Clean up text (no typos in labels, use proper terminology – e.g., use "Network" vs "Server" consistently).
    * Possibly implement any last-minute convenience features that arose from beta feedback (for example, if testers said “I really miss being able to collapse the server tree”, implement that if small).
    * Check accessibility: try navigating with keyboard only. Maybe test with a screen reader (NVDA on Windows or VoiceOver on Mac) to see if basic announcements occur. If not much we can do, at least note it and possibly plan improvements later.
    * Finalize default settings: e.g., default theme dark or light? default highlight color? Ensure defaults make sense for average user. Perhaps have "first run" open the network list to help them get started.
  * *Documentation & Help*:

    * Write a user manual or help file. Could be a Markdown in the repo or a webpage. Cover basics: connecting, common commands (maybe a cheat sheet of IRC commands for newbies), how to use the interface, troubleshooting (like if cannot connect, firewall? etc.), how to use scripts and where to find more.
    * If possible, incorporate a help viewer in app (like a help menu linking to an online doc or opening a local HTML).
    * Document the plugin API for developers if we want external devs to write plugins. Also, an overview of the code architecture in developer docs might attract open-source contributors.
    * Prepare a changelog of sorts, summarizing all the phases improvements.
  * *Test Coverage*: Increase unit test coverage for critical modules. For example, ensure parsing and formatting functions have near 100% coverage (since they are easy to test). Test utility functions (like color code stripper, etc.). It’s harder to test GUI in automated way, but test core logic thoroughly.

    * Possibly test script APIs in isolation (e.g., call a dummy Lua hook and see if it executes correctly).
  * *Beta Feedback Integration*: If we released a beta after Phase 5, collect feedback from users. If something is overwhelmingly requested or a serious issue, address it. This might include UI tweaks (like "add an option for join/part message visibility" – perhaps allow turning off join/part spam display), or feature adjustments (maybe our notification was too basic, we refine it).

    * Be careful not to scope creep too much at this late stage; prioritize critical usability issues or quick wins, and defer bigger new features to post-1.0.
* **Deliverables**:

  * A nearly final version of RustIRC, labeled Release Candidate or 1.0.0-beta2, etc., which has no known major bugs.
  * Test reports or logs showing results of test cases, possibly a summary of any performance improvements achieved (for project records).
  * Full documentation ready for end-users and developers (perhaps hosted on a small website or included in package).
  * A list of known issues (if any) that will not be fixed in 1.0 but are minor or planned for future, for transparency.
  * Possibly packaging scripts ready (though Phase 7 covers final packaging, here we might test an installer to ensure nothing is missing).
* **Milestone Criteria**: Phase 6 is done when:

  * The team can run RustIRC for hours/days in active channels without crashes or memory bloats.
  * All features work as expected and any corner-case bugs found are fixed or deemed negligible.
  * The performance is acceptable on recommended system specs (for instance, runs smoothly on an average laptop with multiple channels).
  * Documentation is complete and understandable.
  * We have confidence that we can release this to the general public and they will have a good experience.

Essentially, at end of Phase 6, we are ready to press "Release 1.0".

* **Risks**: It's possible that some nasty bugs appear late (especially concurrency or memory issues). We should avoid the temptation to add new features in this phase and focus on stability. If something is too problematic (e.g., maybe our plugin system has a flaw), we might choose to disable that part for 1.0 and document it, rather than ship a crashy feature.

  * Time risk: Testing can always find more issues, we must decide a cut-off where we consider it "good enough" given that it's impossible to test every combination. Aim for a balance and potentially plan a minor 1.1 soon after release for any issues that slip through (typical in software).
  * Another risk is burnout or rushing; ensure adequate rest and possibly involve the community in testing to get fresh eyes.

### Phase 7: Release, Distribution, and Maintenance (Weeks 23–24 and beyond)

**Objectives**: Package and distribute RustIRC 1.0 for all target platforms, announce it to users, and set up channels for support/feedback (like a project IRC channel, naturally!). Also plan for ongoing maintenance: issue tracking, future improvements, etc.

* **Tasks**:

  * *Packaging*:

    * **Windows**: Use a tool (WiX, NSIS, or Rust's cargo-bundle if it supports Windows MSI) to create an installer that places RustIRC.exe and associated files (maybe default config, documentation) in Program Files. Include the icon, add Start Menu shortcut. Ensure the installer also bundles any runtime libraries (if we used GTK, we need its DLLs; if Iced with wgpu, include necessary Vulkan/MoltenVK? Actually on Windows likely uses DirectX). Test the installer on a fresh Windows VM to ensure it runs.
    * **macOS**: Create a .app bundle (with Info.plist, icon, etc.) and then a .dmg for distribution. Test on an actual Mac, ensure it launches (on Mac might need to be signed to avoid Gatekeeper warnings; if no developer ID, instruct users to right-click open).
    * **Linux**: Create .deb and .rpm packages if possible (maybe using cargo deb or FPM). Also, prepare a Flatpak (since user specifically desires Flatpak) – write a Flatpak manifest including dependencies. Alternatively, AppImage for a single file binary distribution. Possibly also prepare a package for Arch (submit to AUR).
    * For all packages, include documentation files (README, LICENSE, perhaps a manual page if we have CLI usage).
    * Check dynamic linking: ensure OpenSSL or other libs we used are either bundled or documented as dependency (for deb/rpm specify dependencies).
    * Create checksums for releases and maybe GPG sign them (for integrity).
  * *Website/Announcement*:

    * If not already done, create a simple project website or GitHub Pages that has info about RustIRC, download links, screenshots, etc.
    * Write an announcement post (could be on a blog, or just release notes in GitHub) highlighting features ("RustIRC 1.0 released – modern IRC client in Rust combining the best of mIRC, HexChat, and WeeChat!").
    * Possibly mention it on relevant forums (r/irc on Reddit would likely be interested, given many there talk about clients; also perhaps on IRC channels where IRC client devs hang out).
    * Emphasize that it's open-source and welcome contributions for future.
  * *Support Channels*:

    * Set up an IRC channel for RustIRC support (maybe on Libera or similar) so new users can come ask questions or report issues. Monitor that especially after release.
    * Also use GitHub Issues for bug reports; ensure templates for bug report or feature request are in place to get necessary info.
    * If possible, have some basic FAQ ready (like "why can’t I connect to network X? Possibly need to enable SASL..." etc).
  * *Maintenance Planning*:

    * Identify areas for future improvement (like "add OTR in 1.1" or "improve mobile relay in future"). Create a roadmap for version 1.x and 2.0 ideas. But also be open to user suggestions which might change priorities.
    * Set up CI to continue running for pull requests and maybe nightly builds, to catch regressions early.
    * Possibly set up an automatic crash reporting mechanism (maybe not; could be privacy issue for an IRC client; probably skip this and rely on user reports).
    * Ensure we have means to update the software: if critical bug found, plan for a quick 1.0.1 patch. If we included auto-update, test that it works (or prepare to manually notify users via our channels).
  * *Contributor Engagement*:

    * Now that it's public, encourage others to contribute. Maybe label some easy issues for "good first issue". Provide guidance in CONTRIBUTING.md.
    * If some features didn't make it (like say OTR), maybe someone from community might take it on if there's interest.
    * Keep the momentum by maybe having periodic updates or being responsive to feedback.

* **Deliverables**:

  * RustIRC 1.0 release binaries: e.g., `RustIRC-1.0.0-win64.msi`, `RustIRC-1.0.0.dmg`, `RustIRC-1.0.0-x86_64.AppImage` or .deb/.rpm etc., uploaded to GitHub releases or the project site.
  * Announcement text (could be on GitHub release notes and any other forum post).
  * A live website with documentation and downloads.
  * Established IRC channel (perhaps `#RustIRC` on some network) for live help.
  * Post-release plan document (maybe internal) listing who will handle support issues, how we'll triage, etc., to ensure the project remains healthy.

* **Milestone Criteria**: Phase 7 is successful when:

  * Users can easily download and install RustIRC on their platform of choice.
  * The release is announced and reaches the target audience (IRC users) so that we get some adoption or at least awareness.
  * The infrastructure for maintenance (issue tracker, support channel) is in place and we have responded to initial feedback.
  * The team transitions into maintenance mode, fixing any immediate post-release bugs and merging contributions.

* **Risks**:

  * Packaging can have hiccups (especially codesigning on Mac, or dealing with antivirus false positives on Windows installers). We should allocate time to resolve these (maybe skip signing if not possible, just document how to open).
  * The initial release might have some bugs that escaped testing – be prepared to issue a quick update if something critical arises (maybe we keep version 1.0.1 branch ready to merge fixes).
  * Reception risk: the IRC community can be traditional; some might stick to CLI clients or say "why need new client?" But given HexChat’s uncertain future and interest in Rust, likely many will be excited to try it. We should highlight the unique selling points (modern secure codebase, cross-platform, scriptable, etc.).
  * Maintenance: The project should avoid going stale; ideally we continue with at least minor updates. Engage users to report and maybe even fix issues (Rust community might contribute if they find it interesting).

After Phase 7, RustIRC 1.0 will be out in the wild. The development process will shift to an iterative maintenance cycle (gather feedback for 1.1, etc.). Features that were cut or new ideas (maybe a mobile app or a web GUI for remote use) can be considered then.

---

## Tools and Development Environment

Throughout development, we will utilize various tools to enhance productivity and quality:

* **IDE/Editor**: Primary development in Visual Studio Code with the **rust-analyzer** extension for great Rust language support (autocomplete, go-to-definition, etc.). This helps catch errors early and navigate the codebase efficiently. Team members might also use IntelliJ IDEA with Rust plugin or CLion if they prefer, but VSCode is simple and effective.
* **Version Control**: Git on GitHub for code hosting. We will use a branching strategy (e.g., main branch for stable code, develop branch for integration, feature branches for each major feature or phase). Use pull requests for code reviews if multiple devs.
* **Continuous Integration**: Set up GitHub Actions to run tests on push (check builds on Linux, Windows, Mac, run `cargo test`, maybe `cargo clippy` for linting, and `cargo fmt --check`). This ensures cross-platform compatibility is constantly verified.
* **Issue Tracker**: Use GitHub Issues to log bugs and feature tasks. Possibly use GitHub Projects or a simple Kanban board to track progress in each phase.
* **Communication**: For internal team chat, ironically we could use an IRC channel (on a private network or slack/discord if team prefers). But given the nature, dogfooding our product on an IRC channel might be fun for team discussions.
* **Build and Distribution**: Use Cargo for builds. For cross-compiling releases:

  * On Windows, likely compile natively on Windows (or cross-compile from Linux if setup).
  * On Mac, build on a Mac or Mac CI runner (since cross compiling to Mac is tough).
  * Possibly utilize GitHub Actions CI to build artifacts for all OS and attach to releases (GitHub Actions has Windows and Mac runners which can produce the executables or packages).
* **Testing Tools**: as mentioned, use `cargo test` for unit tests. For integration, maybe a custom test harness or just manual testing. Could use automation tools like Expect or an IRC bot to simulate user input for testing.
* **Profiling**: Use `cargo flamegraph` or `perf` on Linux to profile performance hotspots. On Windows, Visual Studio’s profiler or AMD’s CodeXL could be used if needed.
* **Memory/Leak check**: Valgrind on Linux (though it might flag false positives if memory is held by Rust runtime intentionally). Also consider AddressSanitizer by compiling with certain flags (Rust can use ASan with nightly if needed).
* **Security**: `cargo audit` to check for vulnerable dependencies regularly.
* **Crate Publishing**: We might publish certain parts (like if we made a standalone IRC crate or UI components) to crates.io for others to use, but not critical. The app itself can be distributed via the packages rather than crates.io (since it's not a library primarily, though we can also allow `cargo install rustirc` for those who want to build from source easily).
* **Logging within App**: Use the `log` crate with env\_logger during dev to get debug output. Possibly have a debug console in the app or a mode to redirect logs to a file for troubleshooting user issues (e.g., /debug command to enable logging).
* **Crash handling**: On Windows, ensure we don't just swallow exceptions; maybe use Rust panic hook to log panics to a file so if a user reports crash, we have some data.

The development environment will be mostly standard Rust. We'll ensure all team members have it set up (Rust stable toolchain, etc.). Using clippy (with `#![deny(clippy::all)]` on CI perhaps) to enforce good code style.

We will also maintain compatibility with stable Rust compiler (no nightly-only features needed, to ease contribution).

Finally, ensure that building from source is straightforward for users who prefer (document `cargo build` steps and dependencies like if using GTK, to install dev libraries on Linux etc., though we might static link to simplify).

## Risks and Mitigation

We have identified various risks during planning; here we summarize key risks and how we mitigate them:

* **Scope Creep and Feature Creep**: The plan is ambitious, combining features of three robust clients. Risk is trying to do too much and delaying release indefinitely. Mitigation: We prioritized core functionality first (Phases 1–3) and moved some nice-to-haves to later phases or post-1.0. We will continually re-assess and possibly cut features if timeline slips (for example, if OTR or multi-language scripting is too much, we can release without them and add later).
* **Technical Complexity**: Some parts (GUI, multi-threading, DCC) are complex. Mitigation: Use existing libraries to handle heavy lifting where possible (e.g., rely on crates for GUI elements, use proven architecture patterns). Also, incremental approach (we get a working simple client early in Phase 2/3, so complexity is added on a working base).
* **Cross-Platform Issues**: Ensuring it works on every OS could cause delays. Mitigation: Use cross-platform libraries (Rust crates are often cross-platform). Test early on each target (not just at the end).
* **Community Adoption**: As a new client, risk that users stick to what they know. Mitigation: Our value proposition is strong (modern, secure, combined features). We'll actively engage with communities (maybe get some power users to beta test and evangelize if they like it). Also, being open-source and on GitHub can attract attention in Rust circles (maybe an announcement on /r/rust as well).
* **Maintaining Compatibility with IRC Customs**: If we inadvertently break some IRC convention (like message formatting or assume a specific CHANMODES, etc.), users will notice. We mitigate by extensive testing with known networks and reading the IRC specs carefully.
* **Security**: As with any client, there's risk of misuse or exploits (like malicious formatting sequences). Using Rust already mitigates many memory issues. We must also consider things like not executing arbitrary code except user-loaded scripts, and perhaps sanitize what scripts can do. We'll do a security review (like how to handle message that could be interpreted as commands if echoed, etc., though IRC is text so less an injection risk aside from maybe XSS if we had an HTML view, which we don't).
* **Performance**: Perhaps using a high-level GUI library might be slow for large volumes of text. Mitigation: If performance issues arise, consider optimizations like using a lighter widget for text (maybe drop down to writing directly to a canvas if needed for performance, or limit UI updates frequency).
* **Dependency Issues**: If a crate we rely on has a bug or stops being maintained. Mitigation: Because Rust is open source, we can fork or contribute fixes if needed. Also choose widely used crates. We pinned specific versions for stability.
* **Legal/Policy**: Using GPL for code means any linking with non-GPL might be an issue (if we dynamically link to OpenSSL, etc., but we can use rustls to avoid GPL conflict). Also, distributing on Mac without signing might reduce adoption; ideally we'd get an Apple dev ID, but if not, just instruct how to open.
* **User Data Privacy**: Ensure we are not inadvertently logging sensitive info or sending telemetry. We'll likely have zero telemetry. If we implement an update check, we should make it opt-in or at least transparent. Many IRC users are privacy-conscious, so we align with that by design.
* **Competition and Relevancy**: IRC usage is not as widespread today, so the user base is smaller. However, those who use IRC are dedicated. If we deliver a clearly superior product, we could become the go-to recommendation, which is success for us. Not exactly a risk, but an incentive to polish the user experience to attract even those comfortable with older clients.

By acknowledging these risks and planning around them, we increase our chances of a smooth development cycle and a successful product.

## Conclusion

This plan detailed the vision and execution strategy for **RustIRC**, a next-generation IRC client built in Rust that fuses the strengths of mIRC, HexChat, and WeeChat. We covered everything from core requirements and architectural decisions to a step-by-step roadmap with phases for development. By following this plan, we aim to deliver a secure, feature-rich, and user-friendly IRC client that stands out in 2025 and beyond.

RustIRC will offer longtime IRC enthusiasts a fresh yet familiar experience – powerful scripting, robust file transfers, up-to-date protocol support – all in a performant and safe package. Newcomers will benefit from its ease of use and modern interface, potentially revitalizing interest in IRC as a platform for open communication.

The next steps are to kick off Phase 1 (setup and research validation) and then dive into coding the core in Phase 2. Given the outlined timeline of roughly 5–6 months to beta and a couple more for polish, we anticipate releasing RustIRC 1.0 within a year, with intermediate builds to gather feedback.

With the planning complete, the team is excited to begin implementation. RustIRC has the potential to become a **best-of-breed IRC client**, and with careful execution, it will fulfill that promise. We look forward to building RustIRC and engaging with the IRC community to make it a success.

*For further discussions, progress updates, or contribution opportunities, please refer to the project repository and join our IRC channel (to be announced). Let's bring the legendary IRC experience to the modern era with RustIRC!*
