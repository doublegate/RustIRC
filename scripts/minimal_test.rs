use iced::{Task, Element};

#[derive(Debug, Clone)]
enum Message {}

struct App;

// Testing different trait possibilities
impl App {
    fn new() -> (Self, Task<Message>) {
        (App, Task::none())
    }
    
    fn title(&self) -> String {
        "Test".to_string()
    }
    
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }
    
    fn view(&self) -> Element<Message> {
        iced::widget::text("Test").into()
    }
}

fn main() {
    println!("Testing Iced API structure");
}