//! Animated Loading Spinner - Material Design 3 Style
//! 
//! Custom Canvas widget for smooth circular progress indicator
//! with proper Material Design animations and easing curves.

use crate::themes::material_design_3::MaterialTheme;
use iced::{
    widget::{canvas, Canvas},
    Color, Element, Length, Point, Rectangle, Size, Vector,
    mouse, renderer, time,
};
use std::time::{Duration, Instant};

/// Material Design 3 Animated Spinner
#[derive(Debug)]
pub struct AnimatedSpinner {
    theme: MaterialTheme,
    size: f32,
    stroke_width: f32,
    color: Option<Color>,
    determinate: bool,
    progress: f32, // 0.0 to 1.0 for determinate mode
}

/// Spinner message for animation updates
#[derive(Debug, Clone)]
pub enum SpinnerMessage {
    Tick(Instant),
    ProgressChanged(f32),
}

/// Internal state for spinner animation
#[derive(Debug, Default)]
pub struct SpinnerState {
    animation_start: Option<Instant>,
    rotation: f32,
    sweep_angle: f32,
    sweep_direction: f32,
    indeterminate_cycle: f32,
}

impl AnimatedSpinner {
    /// Create new animated spinner
    pub fn new() -> Self {
        Self {
            theme: MaterialTheme::dark(),
            size: 40.0,
            stroke_width: 4.0,
            color: None,
            determinate: false,
            progress: 0.0,
        }
    }

    /// Set spinner size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.stroke_width = size * 0.1; // 10% of size
        self
    }

    /// Set stroke width
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Set custom color (overrides theme)
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Create determinate spinner with progress
    pub fn determinate(mut self, progress: f32) -> Self {
        self.determinate = true;
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Update progress for determinate spinner
    pub fn set_progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Build spinner element
    pub fn build<Message>(self) -> Element<'static, Message>
    where
        Message: 'static + Clone,
        SpinnerMessage: Into<Message>,
    {
        Canvas::new(self)
            .width(Length::Fixed(self.size))
            .height(Length::Fixed(self.size))
            .into()
    }

    /// Get spinner color
    fn get_color(&self) -> Color {
        self.color.unwrap_or(self.theme.scheme.primary)
    }
}

impl Default for AnimatedSpinner {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message> canvas::Program<Message> for AnimatedSpinner
where
    Message: Clone + 'static,
    SpinnerMessage: Into<Message>,
{
    type State = SpinnerState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (self.size / 2.0) - (self.stroke_width / 2.0);
        let color = self.get_color();

        let geometry = canvas::Geometry::new();
        let mut frame = canvas::Frame::new(bounds.size());

        if self.determinate {
            // Determinate spinner - show progress
            self.draw_determinate(&mut frame, center, radius, color);
        } else {
            // Indeterminate spinner - animated
            self.draw_indeterminate(&mut frame, center, radius, color, state);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        _event: canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        // Animation is driven by tick messages
        (canvas::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl AnimatedSpinner {
    /// Draw determinate progress indicator
    fn draw_determinate(
        &self,
        frame: &mut canvas::Frame,
        center: Point,
        radius: f32,
        color: Color,
    ) {
        // Background circle (track)
        let track_color = Color {
            a: 0.3,
            ..color
        };
        
        frame.stroke(
            &canvas::Path::circle(center, radius),
            canvas::Stroke {
                color: track_color,
                width: self.stroke_width,
                line_cap: canvas::LineCap::Round,
                ..Default::default()
            },
        );

        // Progress arc
        if self.progress > 0.0 {
            let sweep_angle = 2.0 * std::f32::consts::PI * self.progress;
            let start_angle = -std::f32::consts::PI / 2.0; // Start at top

            let path = canvas::Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle,
                    end_angle: start_angle + sweep_angle,
                });
            });

            frame.stroke(
                &path,
                canvas::Stroke {
                    color,
                    width: self.stroke_width,
                    line_cap: canvas::LineCap::Round,
                    ..Default::default()
                },
            );
        }
    }

    /// Draw indeterminate animated spinner
    fn draw_indeterminate(
        &self,
        frame: &mut canvas::Frame,
        center: Point,
        radius: f32,
        color: Color,
        state: &SpinnerState,
    ) {
        // Material Design indeterminate spinner animation
        // The arc sweeps around the circle with varying length

        let rotation = state.rotation;
        let sweep_angle = state.sweep_angle;
        
        // Calculate start and end angles
        let base_start = rotation - std::f32::consts::PI / 2.0;
        let start_angle = base_start;
        let end_angle = base_start + sweep_angle;

        if sweep_angle > 0.01 {
            let path = canvas::Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle,
                    end_angle,
                });
            });

            // Use gradient-like effect by varying opacity
            let alpha = (0.8 + 0.2 * (state.indeterminate_cycle * 4.0).sin()).max(0.6);
            let animated_color = Color { a: alpha, ..color };

            frame.stroke(
                &path,
                canvas::Stroke {
                    color: animated_color,
                    width: self.stroke_width,
                    line_cap: canvas::LineCap::Round,
                    ..Default::default()
                },
            );
        }
    }
}

/// Update spinner animation state
pub fn update_spinner_animation(
    state: &mut SpinnerState,
    now: Instant,
    is_determinate: bool,
) {
    if is_determinate {
        return; // No animation for determinate mode
    }

    // Initialize animation start time
    if state.animation_start.is_none() {
        state.animation_start = Some(now);
    }

    let start_time = state.animation_start.unwrap();
    let elapsed = now.duration_since(start_time).as_secs_f32();

    // Material Design indeterminate animation parameters
    const ROTATION_DURATION: f32 = 2.0; // Full rotation in 2 seconds
    const SWEEP_DURATION: f32 = 1.5;    // Sweep cycle in 1.5 seconds

    // Update rotation (continuous)
    state.rotation = (elapsed / ROTATION_DURATION * 2.0 * std::f32::consts::PI) % (2.0 * std::f32::consts::PI);

    // Update sweep angle (oscillating)
    let sweep_cycle = (elapsed / SWEEP_DURATION) % 1.0;
    state.indeterminate_cycle = sweep_cycle;

    // Use Material Design easing curve for sweep
    let normalized_cycle = sweep_cycle * 2.0 * std::f32::consts::PI;
    let eased_cycle = (1.0 - (normalized_cycle).cos()) / 2.0;
    
    // Sweep angle varies from small to large
    const MIN_SWEEP: f32 = 0.1;
    const MAX_SWEEP: f32 = 0.75 * 2.0 * std::f32::consts::PI;
    
    state.sweep_angle = MIN_SWEEP + (MAX_SWEEP - MIN_SWEEP) * eased_cycle;
}

/// Create subscription for spinner animation
pub fn spinner_subscription<Message>() -> iced::Subscription<Message>
where
    Message: 'static + Clone,
    SpinnerMessage: Into<Message>,
{
    time::every(Duration::from_millis(16)) // ~60fps
        .map(SpinnerMessage::Tick)
        .map(Into::into)
}

/// Convenience functions for common spinner configurations
pub fn small_spinner<Message>() -> AnimatedSpinner {
    AnimatedSpinner::new().size(24.0)
}

pub fn medium_spinner<Message>() -> AnimatedSpinner {
    AnimatedSpinner::new().size(40.0)
}

pub fn large_spinner<Message>() -> AnimatedSpinner {
    AnimatedSpinner::new().size(64.0)
}

pub fn progress_spinner<Message>(progress: f32) -> AnimatedSpinner {
    AnimatedSpinner::new()
        .size(40.0)
        .determinate(progress)
}

pub fn colored_spinner<Message>(color: Color) -> AnimatedSpinner {
    AnimatedSpinner::new()
        .size(40.0)
        .color(color)
}