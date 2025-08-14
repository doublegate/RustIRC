use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

struct App {
    channels: Vec<Channel>,
    active_channel: usize,
    input: String,
    input_cursor: usize,
    scroll_offset: usize,
}

struct Channel {
    name: String,
    messages: Vec<String>,
    users: Vec<String>,
    unread: bool,
}

impl App {
    fn new() -> Self {
        let mut channels = vec![
            Channel {
                name: "Server".to_string(),
                messages: vec![
                    "* Connected to irc.libera.chat".to_string(),
                    "* Your host is irc.libera.chat, running version InspIRCd-3".to_string(),
                    "* This server was created 09:32:54 Oct 12 2024".to_string(),
                ],
                users: vec![],
                unread: false,
            },
            Channel {
                name: "#rust".to_string(),
                messages: vec![
                    "* Now talking in #rust".to_string(),
                    "* Topic: Rust Programming Language | https://rust-lang.org".to_string(),
                    "<alice> Welcome to #rust!".to_string(),
                    "<bob> Has anyone tried the new async traits?".to_string(),
                ],
                users: vec![
                    "@ops".to_string(),
                    "+alice".to_string(),
                    "bob".to_string(),
                    "charlie".to_string(),
                    "dave".to_string(),
                ],
                unread: true,
            },
            Channel {
                name: "#rustirc".to_string(),
                messages: vec![
                    "* Now talking in #rustirc".to_string(),
                    "* Topic: RustIRC Development - Modern IRC Client".to_string(),
                    "<dev> Testing the TUI prototype".to_string(),
                ],
                users: vec!["@dev".to_string(), "tester".to_string()],
                unread: false,
            },
        ];

        // Add performance test messages
        for i in 0..1000 {
            channels[1].messages.push(format!(
                "<user{}> Test message #{} - Checking TUI performance with scrollback",
                i % 20,
                i
            ));
        }

        Self {
            channels,
            active_channel: 0,
            input: String::new(),
            input_cursor: 0,
            scroll_offset: 0,
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Tab => {
                self.active_channel = (self.active_channel + 1) % self.channels.len();
                self.channels[self.active_channel].unread = false;
                self.scroll_offset = 0;
            }
            KeyCode::Char(c) => {
                self.input.insert(self.input_cursor, c);
                self.input_cursor += 1;
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    self.input.remove(self.input_cursor - 1);
                    self.input_cursor -= 1;
                }
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
                }
            }
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    self.channels[self.active_channel]
                        .messages
                        .push(format!("<you> {}", self.input));
                    self.input.clear();
                    self.input_cursor = 0;
                    self.scroll_offset = 0;
                }
            }
            KeyCode::PageUp => {
                let channel = &self.channels[self.active_channel];
                if self.scroll_offset < channel.messages.len() {
                    self.scroll_offset = (self.scroll_offset + 10).min(channel.messages.len());
                }
            }
            KeyCode::PageDown => {
                if self.scroll_offset > 10 {
                    self.scroll_offset -= 10;
                } else {
                    self.scroll_offset = 0;
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') if app.input.is_empty() => {
                            return Ok(());
                        }
                        code => app.on_key(code),
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),     // Status bar
            Constraint::Min(0),        // Main content
            Constraint::Length(3),     // Input bar
        ])
        .split(f.area());

    // Status bar with channel tabs
    let channel_list: Vec<Span> = app
        .channels
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            let style = if i == app.active_channel {
                Style::default().fg(Color::Black).bg(Color::White)
            } else if ch.unread {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            if i > 0 {
                Span::styled(format!(" {} ", ch.name), style)
            } else {
                Span::styled(format!("{} ", ch.name), style)
            }
        })
        .collect();

    let status = Paragraph::new(Line::from(channel_list))
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Left);
    f.render_widget(status, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(chunks[1]);

    // Messages area
    let channel = &app.channels[app.active_channel];
    let messages_len = channel.messages.len();
    let display_messages = if app.scroll_offset >= messages_len {
        &channel.messages[..]
    } else {
        let end = messages_len - app.scroll_offset;
        let start = end.saturating_sub(main_chunks[0].height as usize);
        &channel.messages[start..end]
    };

    let messages: Vec<ListItem> = display_messages
        .iter()
        .map(|msg| {
            let style = if msg.starts_with('*') {
                Style::default().fg(Color::Yellow)
            } else if msg.starts_with('<') {
                if msg.starts_with("<you>") {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                }
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(msg.as_str()).style(style)
        })
        .collect();

    let messages_widget = List::new(messages).block(
        Block::default()
            .borders(Borders::RIGHT)
            .title(format!(" {} ", channel.name)),
    );
    f.render_widget(messages_widget, main_chunks[0]);

    // User list
    if !channel.users.is_empty() {
        let users: Vec<ListItem> = channel
            .users
            .iter()
            .map(|user| {
                let style = if user.starts_with('@') {
                    Style::default().fg(Color::Red)
                } else if user.starts_with('+') {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(user.as_str()).style(style)
            })
            .collect();

        let users_widget = List::new(users).block(
            Block::default()
                .borders(Borders::NONE)
                .title(" Users "),
        );
        f.render_widget(users_widget, main_chunks[1]);
    }

    // Input bar
    let input_widget = Paragraph::new(app.input.as_str())
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Input (Tab: switch channel, PgUp/PgDn: scroll, Esc: quit) "),
        );
    f.render_widget(input_widget, chunks[2]);

    // Cursor position
    f.set_cursor_position((chunks[2].x + app.input_cursor as u16 + 1, chunks[2].y + 1));
}