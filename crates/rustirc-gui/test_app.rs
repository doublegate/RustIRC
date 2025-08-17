use iced::{Element, Task};

fn main() {
    println!("Testing Iced API");
}

struct TestApp;

impl iced::Program for TestApp {
    type Message = ();
    type Theme = iced::Theme;
    type State = ();
    
    fn title(&self, _state: &Self::State) -> String {
        "Test".to_string()
    }
    
    fn update(&self, _state: &mut Self::State, _message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }
    
    fn view(&self, _state: &Self::State) -> Element<Self::Message> {
        iced::widget::text("Hello").into()
    }
}
