use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Container};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};

pub fn main() -> iced::Result {
    IrcPrototype::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    SendMessage,
    TabSelected(usize),
}

struct IrcPrototype {
    channels: Vec<Channel>,
    active_tab: usize,
    input_value: String,
}

struct Channel {
    name: String,
    messages: Vec<String>,
    users: Vec<String>,
}

impl Application for IrcPrototype {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut app = IrcPrototype {
            channels: vec![
                Channel {
                    name: "Server".to_string(),
                    messages: vec![
                        "* Connected to irc.libera.chat".to_string(),
                        "* Your host is irc.libera.chat".to_string(),
                    ],
                    users: vec![],
                },
                Channel {
                    name: "#rust".to_string(),
                    messages: vec![
                        "<alice> Welcome to #rust!".to_string(),
                        "<bob> Check out the latest RFC".to_string(),
                    ],
                    users: vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()],
                },
                Channel {
                    name: "#rustirc".to_string(),
                    messages: vec![
                        "* Topic: RustIRC Development".to_string(),
                        "<dev> Testing the new client".to_string(),
                    ],
                    users: vec!["dev".to_string(), "tester".to_string()],
                },
            ],
            active_tab: 0,
            input_value: String::new(),
        };

        // Add performance test data - 10k lines
        for i in 0..10000 {
            app.channels[1].messages.push(format!(
                "<user{}> Performance test message #{} - Testing scrollback and rendering performance with large message history",
                i % 50, i
            ));
        }

        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("RustIRC - Iced Prototype")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::SendMessage => {
                if !self.input_value.is_empty() {
                    self.channels[self.active_tab]
                        .messages
                        .push(format!("<you> {}", self.input_value));
                    self.input_value.clear();
                }
            }
            Message::TabSelected(index) => {
                self.active_tab = index;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let channel_tabs = row(
            self.channels
                .iter()
                .enumerate()
                .map(|(i, channel)| {
                    let is_active = i == self.active_tab;
                    let tab_button = button(text(&channel.name).size(14))
                        .padding([8, 16])
                        .style(if is_active {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::TabSelected(i));
                    
                    Element::from(tab_button)
                })
                .collect(),
        )
        .spacing(4)
        .padding(4);

        let active_channel = &self.channels[self.active_tab];

        let message_list = scrollable(
            Column::new()
                .extend(
                    active_channel
                        .messages
                        .iter()
                        .map(|msg| {
                            let styled_msg = if msg.starts_with('<') {
                                // User message
                                text(msg).size(14)
                            } else if msg.starts_with('*') {
                                // System message
                                text(msg).size(14).style(iced::theme::Text::Color(
                                    iced::Color::from_rgb(0.5, 0.5, 0.5),
                                ))
                            } else {
                                text(msg).size(14)
                            };
                            Element::from(styled_msg)
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(2)
                .padding(8),
        )
        .height(Length::Fill)
        .width(Length::FillPortion(3));

        let user_list = if !active_channel.users.is_empty() {
            Some(
                scrollable(
                    Column::new()
                        .extend(
                            active_channel
                                .users
                                .iter()
                                .map(|user| Element::from(text(user).size(14)))
                                .collect::<Vec<_>>(),
                        )
                        .spacing(2)
                        .padding(8),
                )
                .height(Length::Fill)
                .width(Length::FillPortion(1)),
            )
        } else {
            None
        };

        let input_bar = row![
            text_input("Type a message...", &self.input_value)
                .on_input(Message::InputChanged)
                .on_submit(Message::SendMessage)
                .padding(8)
                .size(14)
                .width(Length::Fill),
            button("Send")
                .on_press(Message::SendMessage)
                .padding([8, 16])
        ]
        .spacing(8)
        .padding(8);

        let main_content = if let Some(users) = user_list {
            row![message_list, users]
        } else {
            row![message_list]
        };

        let content = column![channel_tabs, main_content, input_bar]
            .spacing(0)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}