//! Virtual scrolling widget for efficient rendering of large lists
//!
//! Implements virtual scrolling to handle thousands of messages efficiently
//! by only rendering items currently visible in the viewport.

use iced::{Element, Length, Task, Rectangle, Size, Point};
use iced::widget::{container, scrollable, Column};
use std::collections::VecDeque;
use std::fmt::Debug;

/// Virtual scrolling widget for large item lists
pub struct VirtualScroll<T, Message> {
    items: VecDeque<T>,
    viewport_height: f32,
    item_height: f32,
    scroll_offset: f32,
    visible_items: usize,
    overscan: usize,
    render_item: Box<dyn Fn(&T, usize) -> Element<Message>>,
}

impl<T, Message> VirtualScroll<T, Message> 
where 
    T: Clone + Debug,
    Message: Clone + 'static,
{
    /// Create a new virtual scroll widget
    pub fn new(
        items: VecDeque<T>,
        item_height: f32,
        render_item: impl Fn(&T, usize) -> Element<Message> + 'static,
    ) -> Self {
        Self {
            items,
            viewport_height: 400.0, // Default height
            item_height,
            scroll_offset: 0.0,
            visible_items: 0,
            overscan: 5, // Render 5 extra items above/below viewport
            render_item: Box::new(render_item),
        }
    }
    
    /// Update the item list
    pub fn update_items(&mut self, items: VecDeque<T>) {
        self.items = items;
        self.update_visible_range();
    }
    
    /// Add item to the end
    pub fn push_item(&mut self, item: T) {
        self.items.push_back(item);
        self.update_visible_range();
    }
    
    /// Add item to the beginning
    pub fn push_front(&mut self, item: T) {
        self.items.push_front(item);
        self.update_visible_range();
    }
    
    /// Remove oldest items to maintain size limit
    pub fn trim_to_size(&mut self, max_items: usize) {
        while self.items.len() > max_items {
            self.items.pop_front();
        }
        self.update_visible_range();
    }
    
    /// Set viewport height
    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_height = height;
        self.update_visible_range();
    }
    
    /// Set scroll position
    pub fn set_scroll_offset(&mut self, offset: f32) {
        self.scroll_offset = offset.max(0.0);
        self.update_visible_range();
    }
    
    /// Scroll to specific item index
    pub fn scroll_to_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.scroll_offset = (index as f32 * self.item_height)
                .min(self.max_scroll_offset());
            self.update_visible_range();
        }
    }
    
    /// Scroll to bottom (most recent)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
        self.update_visible_range();
    }
    
    /// Check if scrolled to bottom
    pub fn is_at_bottom(&self) -> bool {
        let max_offset = self.max_scroll_offset();
        (self.scroll_offset - max_offset).abs() < self.item_height
    }
    
    /// Get total content height
    pub fn total_height(&self) -> f32 {
        self.items.len() as f32 * self.item_height
    }
    
    /// Get maximum scroll offset
    pub fn max_scroll_offset(&self) -> f32 {
        (self.total_height() - self.viewport_height).max(0.0)
    }
    
    /// Update visible item range
    fn update_visible_range(&mut self) {
        let visible_count = (self.viewport_height / self.item_height).ceil() as usize;
        self.visible_items = visible_count + (2 * self.overscan);
    }
    
    /// Get visible item range
    pub fn visible_range(&self) -> (usize, usize) {
        let start_index = ((self.scroll_offset / self.item_height) as usize)
            .saturating_sub(self.overscan);
            
        let end_index = (start_index + self.visible_items)
            .min(self.items.len());
            
        (start_index, end_index)
    }
    
    /// Get visible items for rendering
    pub fn visible_items(&self) -> impl Iterator<Item = (usize, &T)> {
        let (start, end) = self.visible_range();
        self.items
            .iter()
            .enumerate()
            .skip(start)
            .take(end - start)
    }
    
    /// Render the virtual scroll widget
    pub fn view(&self) -> Element<Message> {
        let (start_index, _) = self.visible_range();
        
        // Create spacer for items above viewport
        let top_spacer_height = start_index as f32 * self.item_height;
        
        // Render visible items
        let visible_elements: Vec<Element<Message>> = self
            .visible_items()
            .map(|(index, item)| (self.render_item)(item, index))
            .collect();
        
        // Calculate bottom spacer height
        let rendered_count = visible_elements.len();
        let bottom_spacer_height = ((self.items.len() - start_index - rendered_count) as f32
            * self.item_height).max(0.0);
        
        // Build the scrollable content with spacers
        let mut column = Column::new();
        
        // Top spacer
        if top_spacer_height > 0.0 {
            column = column.push(
                container(iced::widget::text(""))
                    .height(Length::Fixed(top_spacer_height))
            );
        }
        
        // Visible items
        for element in visible_elements {
            column = column.push(element);
        }
        
        // Bottom spacer
        if bottom_spacer_height > 0.0 {
            column = column.push(
                container(iced::widget::text(""))
                    .height(Length::Fixed(bottom_spacer_height))
            );
        }
        
        scrollable(column)
            .height(Length::Fixed(self.viewport_height))
            .into()
    }
}

/// Performance statistics for virtual scrolling
#[derive(Debug, Clone)]
pub struct VirtualScrollStats {
    pub total_items: usize,
    pub visible_items: usize,
    pub rendered_items: usize,
    pub memory_saved_ratio: f32,
    pub scroll_position: f32,
    pub viewport_efficiency: f32,
}

impl<T, Message> VirtualScroll<T, Message> 
where 
    T: Clone + Debug,
    Message: Clone + 'static,
{
    /// Get performance statistics
    pub fn stats(&self) -> VirtualScrollStats {
        let (start, end) = self.visible_range();
        let rendered_items = end - start;
        let memory_saved_ratio = if self.items.len() > 0 {
            1.0 - (rendered_items as f32 / self.items.len() as f32)
        } else {
            0.0
        };
        
        let viewport_efficiency = if rendered_items > 0 {
            let truly_visible = (self.viewport_height / self.item_height) as usize;
            truly_visible as f32 / rendered_items as f32
        } else {
            0.0
        };
        
        VirtualScrollStats {
            total_items: self.items.len(),
            visible_items: (self.viewport_height / self.item_height) as usize,
            rendered_items,
            memory_saved_ratio,
            scroll_position: self.scroll_offset / self.max_scroll_offset().max(1.0),
            viewport_efficiency,
        }
    }
}

/// Virtual scroll configuration
#[derive(Debug, Clone)]
pub struct VirtualScrollConfig {
    pub item_height: f32,
    pub viewport_height: f32,
    pub overscan_count: usize,
    pub auto_scroll_to_bottom: bool,
    pub smooth_scrolling: bool,
    pub scroll_sensitivity: f32,
}

impl Default for VirtualScrollConfig {
    fn default() -> Self {
        Self {
            item_height: 20.0,
            viewport_height: 400.0,
            overscan_count: 5,
            auto_scroll_to_bottom: true,
            smooth_scrolling: true,
            scroll_sensitivity: 1.0,
        }
    }
}

/// Optimized virtual scroll for IRC messages
pub type MessageVirtualScroll<T, Message> = VirtualScroll<T, Message>;

impl<T, Message> MessageVirtualScroll<T, Message> 
where 
    T: Clone + Debug,
    Message: Clone + 'static,
{
    /// Create optimized for IRC messages
    pub fn for_messages(
        items: VecDeque<T>,
        render_item: impl Fn(&T, usize) -> Element<Message> + 'static,
    ) -> Self {
        Self::new(items, 22.0, render_item) // Slightly taller for IRC messages
    }
    
    /// Handle new message with auto-scroll logic
    pub fn handle_new_message(&mut self, message: T, auto_scroll: bool) {
        let was_at_bottom = self.is_at_bottom();
        self.push_item(message);
        
        // Auto-scroll if we were at bottom or auto_scroll is forced
        if was_at_bottom || auto_scroll {
            self.scroll_to_bottom();
        }
    }
    
    /// Batch add multiple messages efficiently
    pub fn batch_add_messages(&mut self, messages: Vec<T>, auto_scroll: bool) {
        let was_at_bottom = self.is_at_bottom();
        
        for message in messages {
            self.items.push_back(message);
        }
        
        self.update_visible_range();
        
        if was_at_bottom || auto_scroll {
            self.scroll_to_bottom();
        }
    }
}