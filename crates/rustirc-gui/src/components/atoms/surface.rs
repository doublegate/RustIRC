//! Material Design 3 Surface component

use iced::{widget::container, Background, Border, Color, Element, Length, Renderer, Theme};

use crate::themes::material_design_3::{ElevationLevel, MaterialTheme};

/// Material Design 3 Surface component
/// Surfaces are foundational elements that affect how components are perceived
#[derive()]
pub struct MaterialSurface<'a, Message, T = iced::Theme, R = iced::Renderer> {
    content: Element<'a, Message, T, R>,
    theme: MaterialTheme,
    elevation: ElevationLevel,
    surface_type: SurfaceType,
    width: Length,
    height: Length,
    padding: u16,
}

/// Surface type variants following Material Design 3 guidelines
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceType {
    /// Primary surface color
    Surface,
    /// Surface with tonal variations
    SurfaceVariant,
    /// Container surfaces for grouping content
    SurfaceContainer,
    /// Low elevation container
    SurfaceContainerLow,
    /// High elevation container  
    SurfaceContainerHigh,
    /// Highest elevation container
    SurfaceContainerHighest,
    /// Inverse surface for high contrast
    InverseSurface,
}

impl<'a, Message, T, R> MaterialSurface<'a, Message, T, R>
where
    R: iced::advanced::Renderer,
    T: iced::widget::container::Catalog,
{
    pub fn new(content: impl Into<Element<'a, Message, T, R>>) -> Self {
        Self {
            content: content.into(),
            theme: MaterialTheme::default(),
            elevation: ElevationLevel::Level0,
            surface_type: SurfaceType::Surface,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: 0,
        }
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn elevation(mut self, elevation: ElevationLevel) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn surface_type(mut self, surface_type: SurfaceType) -> Self {
        self.surface_type = surface_type;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn view(self) -> Element<'a, Message, T, R> {
        let surface_color = self.get_surface_color();

        container(self.content)
            .width(self.width)
            .height(self.height)
            .padding(self.padding)
            .style(move |_theme: &T| {
                let shadow_offset = match self.elevation {
                    ElevationLevel::Level0 => 0.0,
                    ElevationLevel::Level1 => 1.0,
                    ElevationLevel::Level2 => 3.0,
                    ElevationLevel::Level3 => 6.0,
                    ElevationLevel::Level4 => 8.0,
                    ElevationLevel::Level5 => 12.0,
                };

                let shadow_blur = shadow_offset * 2.0;

                container::Style {
                    background: Some(Background::Color(surface_color)),
                    border: Border::default(),
                    shadow: if shadow_offset > 0.0 {
                        iced::Shadow {
                            color: Color::BLACK.scale_alpha(0.15),
                            offset: iced::Vector::new(0.0, shadow_offset),
                            blur_radius: shadow_blur,
                        }
                    } else {
                        iced::Shadow::default()
                    },
                    ..Default::default()
                }
            })
            .into()
    }

    fn get_surface_color(&self) -> Color {
        match self.surface_type {
            SurfaceType::Surface => self.theme.scheme.surface,
            SurfaceType::SurfaceVariant => self.theme.scheme.surface_variant,
            SurfaceType::SurfaceContainer => self.theme.scheme.surface_container,
            SurfaceType::SurfaceContainerLow => self.theme.scheme.surface_container_low,
            SurfaceType::SurfaceContainerHigh => self.theme.scheme.surface_container_high,
            SurfaceType::SurfaceContainerHighest => self.theme.scheme.surface_container_highest,
            SurfaceType::InverseSurface => self.theme.scheme.inverse_surface,
        }
    }
}

/// Helper functions for creating common surface configurations
impl<'a, Message, T, R> MaterialSurface<'a, Message, T, R>
where
    R: iced::advanced::Renderer,
    T: iced::widget::container::Catalog,
{
    /// Creates a card surface with elevation
    pub fn card(content: impl Into<Element<'a, Message, T, R>>) -> Self {
        Self::new(content)
            .surface_type(SurfaceType::SurfaceContainer)
            .elevation(ElevationLevel::Level1)
            .padding(16)
    }

    /// Creates a dialog surface with high elevation
    pub fn dialog(content: impl Into<Element<'a, Message, T, R>>) -> Self {
        Self::new(content)
            .surface_type(SurfaceType::SurfaceContainerHigh)
            .elevation(ElevationLevel::Level3)
            .padding(24)
    }

    /// Creates a bottom sheet surface
    pub fn bottom_sheet(content: impl Into<Element<'a, Message, T, R>>) -> Self {
        Self::new(content)
            .surface_type(SurfaceType::SurfaceContainerLow)
            .elevation(ElevationLevel::Level1)
            .padding(16)
    }

    /// Creates a navbar surface
    pub fn navbar(content: impl Into<Element<'a, Message, T, R>>) -> Self {
        Self::new(content)
            .surface_type(SurfaceType::SurfaceContainer)
            .elevation(ElevationLevel::Level2)
            .padding(8)
    }
}
