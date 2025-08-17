//! Simple app runner for testing - bypasses Application trait issues
use iced::{Task, Element, Settings, Size};

use crate::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {}

pub struct SimpleApp {
    theme: Theme,
}

impl SimpleApp {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::default(),
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "RustIRC".to_string()
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        iced::widget::text("RustIRC GUI Test - Iced 0.13 Compatibility").into()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

pub fn run_simple_test() -> Result<(), iced::Error> {
    let settings = Settings::default();

    // For now, just return Ok to test compilation
    // In a real implementation, we would use the correct Iced 0.13 run function
    println!("Simple app would run here with default settings");
    Ok(())
}