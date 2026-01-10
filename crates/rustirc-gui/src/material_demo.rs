//! Material Design 3 Components Demo
//!
//! This module showcases all the Material Design 3 components
//! that have been implemented for the RustIRC GUI.

use crate::components::molecules::bottom_navigation::BottomNavigationItem;
use crate::components::{
    ButtonVariant, ChipVariant, MaterialAppBar, MaterialBottomNavigation, MaterialButton,
    MaterialCard, MaterialChip, MaterialInput, MaterialListItem, MaterialSearchBar, MaterialText,
    TypographyVariant,
};
use crate::themes::material_design_3::MaterialTheme;
use iced::{
    widget::{column, container, row, scrollable, text, Space},
    Element, Length, Task, Theme,
};

/// Material Design 3 Demo State
#[derive(Default)]
struct MaterialDemoState {
    theme: MaterialTheme,
    text_input_value: String,
    search_value: String,
    selected_chip: Option<usize>,
}

#[derive(Debug, Clone)]
enum Message {
    ButtonPressed(String),
    TextInputChanged(String),
    SearchChanged(String),
    ChipSelected(usize),
    NavigationSelected(usize),
}

/// Run the Material Design 3 demo using Iced functional API
pub fn run() -> iced::Result {
    iced::application(MaterialDemoState::default, update, view)
        .title("RustIRC - Material Design 3 Demo")
        .theme(theme)
        .run()
}

fn update(state: &mut MaterialDemoState, message: Message) -> Task<Message> {
    match message {
        Message::ButtonPressed(label) => {
            println!("Button pressed: {label}");
        }
        Message::TextInputChanged(value) => {
            state.text_input_value = value;
        }
        Message::SearchChanged(value) => {
            state.search_value = value;
        }
        Message::ChipSelected(index) => {
            state.selected_chip = Some(index);
        }
        Message::NavigationSelected(index) => {
            println!("Navigation item {index} selected");
        }
    }
    Task::none()
}

fn view(state: &MaterialDemoState) -> Element<'_, Message> {
    // Create the scrollable content - wrap in container with no vertical fill
    let scroll_content = container(
        column![
            // Typography Section
            typography_section(&state.theme),
            Space::new().height(20),
            // Buttons Section
            buttons_section(&state.theme),
            Space::new().height(20),
            // Input Section
            input_section(&state.theme, &state.text_input_value),
            Space::new().height(20),
            // Chips Section
            chips_section(&state.theme, state.selected_chip),
            Space::new().height(20),
            // Cards Section
            cards_section(&state.theme),
            Space::new().height(20),
            // Search Bar
            search_section(&state.theme, &state.search_value),
            Space::new().height(20),
            // List Items
            list_items_section(&state.theme),
            Space::new().height(20),
            // Message Bubbles
            message_bubbles_section(&state.theme),
            Space::new().height(100), // Extra space for bottom navigation
        ]
        .padding(20)
        .spacing(10)
    )
    .width(Length::Fill)  // Width can be Fill (horizontal axis)
    .height(Length::Shrink); // Explicitly set height to Shrink

    // Build the main layout with proper constraints
    let main_content = column![
        // App Bar (fixed at top)
        MaterialAppBar::new("Material Design 3 Demo")
            .theme(state.theme.clone())
            .view(),
        // Scrollable content area - set explicit height
        scrollable(scroll_content).height(Length::Fill), // The scrollable itself can fill height
        // Bottom Navigation (fixed at bottom)
        MaterialBottomNavigation::new(vec![
            BottomNavigationItem::new("home", "Home", Message::NavigationSelected(0)),
            BottomNavigationItem::new("search", "Search", Message::NavigationSelected(1)),
            BottomNavigationItem::new("settings", "Settings", Message::NavigationSelected(2)),
        ])
        .theme(state.theme.clone())
        .selected_index(0)
        .view(),
    ];

    container(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn theme(_state: &MaterialDemoState) -> Theme {
    Theme::Dark // Base theme, Material colors are applied through components
}

fn typography_section(theme: &MaterialTheme) -> Element<'_, Message> {
    column![
        MaterialText::new("Typography")
            .variant(TypographyVariant::HeadlineLarge)
            .theme(theme.clone())
            .build(),
        MaterialText::new("Display Large")
            .variant(TypographyVariant::DisplayLarge)
            .theme(theme.clone())
            .build(),
        MaterialText::new("Headline Medium")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),
        MaterialText::new("Title Large")
            .variant(TypographyVariant::TitleLarge)
            .theme(theme.clone())
            .build(),
        MaterialText::new("Body Large - Lorem ipsum dolor sit amet")
            .variant(TypographyVariant::BodyLarge)
            .theme(theme.clone())
            .build(),
        MaterialText::new("Label Medium")
            .variant(TypographyVariant::LabelMedium)
            .theme(theme.clone())
            .build(),
    ]
    .spacing(10)
    .into()
}

fn buttons_section(theme: &MaterialTheme) -> Element<'_, Message> {
    column![
        MaterialText::new("Buttons")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),
        row![
            MaterialButton::new("Filled")
                .variant(ButtonVariant::Filled)
                .theme(theme.clone())
                .on_press(Message::ButtonPressed("Filled".to_string()))
                .build(),
            MaterialButton::new("Outlined")
                .variant(ButtonVariant::Outlined)
                .theme(theme.clone())
                .on_press(Message::ButtonPressed("Outlined".to_string()))
                .build(),
            MaterialButton::new("Text")
                .variant(ButtonVariant::Text)
                .theme(theme.clone())
                .on_press(Message::ButtonPressed("Text".to_string()))
                .build(),
            MaterialButton::new("Tonal")
                .variant(ButtonVariant::FilledTonal)
                .theme(theme.clone())
                .on_press(Message::ButtonPressed("Tonal".to_string()))
                .build(),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .into()
}

fn input_section<'a>(theme: &'a MaterialTheme, value: &'a str) -> Element<'a, Message> {
    column![
        MaterialText::new("Input Fields")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),
        MaterialInput::new("Enter text...", value)
            .theme(theme.clone())
            .on_input(Message::TextInputChanged)
            .view(),
        MaterialInput::new("Disabled input", "Cannot edit this")
            .theme(theme.clone())
            .enabled(false)
            .view(),
    ]
    .spacing(10)
    .into()
}

fn chips_section(theme: &MaterialTheme, selected: Option<usize>) -> Element<'_, Message> {
    column![
        MaterialText::new("Chips")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),
        row![
            MaterialChip::new("Filter Chip")
                .variant(ChipVariant::Filter)
                .selected(selected == Some(0))
                .theme(theme.clone())
                .on_press(Message::ChipSelected(0))
                .view(),
            MaterialChip::new("Input Chip")
                .variant(ChipVariant::Input)
                .selected(selected == Some(1))
                .theme(theme.clone())
                .on_press(Message::ChipSelected(1))
                .view(),
            MaterialChip::new("Suggestion")
                .variant(ChipVariant::Suggestion)
                .selected(selected == Some(2))
                .theme(theme.clone())
                .on_press(Message::ChipSelected(2))
                .view(),
            MaterialChip::new("Assist Chip")
                .variant(ChipVariant::Assist)
                .selected(selected == Some(3))
                .theme(theme.clone())
                .on_press(Message::ChipSelected(3))
                .view(),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .into()
}

fn cards_section(theme: &MaterialTheme) -> Element<'_, Message> {
    column![
        MaterialText::new("Cards")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),

        // MaterialCard needs to wrap content
        MaterialCard::new(
            column![
                text("Card Title").size(18),
                text("Card subtitle with additional information").size(14),
                text("This is the main content of the card. It can contain multiple lines of text and other elements.").size(12)
            ]
            .spacing(8)
        )
        .theme(theme.clone())
        .view(),
    ]
    .spacing(10)
    .into()
}

fn search_section<'a>(theme: &'a MaterialTheme, value: &'a str) -> Element<'a, Message> {
    column![
        MaterialText::new("Search Bar")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),
        MaterialSearchBar::new("Search...", value)
            .theme(theme.clone())
            .on_input(Message::SearchChanged)
            .view(),
    ]
    .spacing(10)
    .into()
}

fn list_items_section(theme: &MaterialTheme) -> Element<'_, Message> {
    column![
        MaterialText::new("List Items")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),
        MaterialListItem::new("List Item 1")
            .secondary_text("With subtitle")
            .theme(theme.clone())
            .view(),
        MaterialListItem::new("List Item 2")
            .secondary_text("Another item with more details")
            .leading_content(crate::components::molecules::list_item::ListLeading::Icon(
                "folder".to_string()
            ))
            .theme(theme.clone())
            .view(),
        MaterialListItem::new("List Item 3")
            .secondary_text("With trailing content")
            .trailing_content(crate::components::molecules::list_item::ListTrailing::Text(
                "12:34".to_string()
            ))
            .theme(theme.clone())
            .view(),
    ]
    .spacing(5)
    .into()
}

fn message_bubbles_section(theme: &MaterialTheme) -> Element<'_, Message> {
    column![
        MaterialText::new("Message Bubbles")
            .variant(TypographyVariant::HeadlineMedium)
            .theme(theme.clone())
            .build(),
        // Message bubble demo using MaterialText with background styling
        MaterialText::new("[Alice] 12:34 PM: Hello! This is an incoming message.")
            .variant(TypographyVariant::BodyLarge)
            .theme(theme.clone())
            .build(),
        MaterialText::new("[You] 12:35 PM: This is my reply to the message above.")
            .variant(TypographyVariant::BodyLarge)
            .theme(theme.clone())
            .build(),
    ]
    .spacing(10)
    .into()
}
