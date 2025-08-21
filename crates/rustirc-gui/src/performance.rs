//! Performance optimization systems for RustIRC GUI
//!
//! Includes message batching, dirty region tracking, and frame rate limiting
//! to ensure smooth operation even under heavy IRC traffic.

use crate::state::{DisplayMessage, MessageType};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Message batching system for smooth UI updates
pub struct MessageBatcher {
    pending_messages: HashMap<String, Vec<DisplayMessage>>, // tab_id -> messages
    batch_size_limit: usize,
    time_limit: Duration,
    last_update: Instant,
    update_callback: Option<Box<dyn Fn(&HashMap<String, Vec<DisplayMessage>>) + Send + Sync>>,
}

impl MessageBatcher {
    /// Create a new message batcher
    pub fn new() -> Self {
        Self {
            pending_messages: HashMap::new(),
            batch_size_limit: 100, // Max messages per batch
            time_limit: Duration::from_millis(250), // Max delay before forcing update
            last_update: Instant::now(),
            update_callback: None,
        }
    }
    
    /// Configure batch parameters
    pub fn configure(&mut self, batch_size: usize, time_limit: Duration) {
        self.batch_size_limit = batch_size;
        self.time_limit = time_limit;
    }
    
    /// Set update callback for when batches are ready
    pub fn set_update_callback<F>(&mut self, callback: F) 
    where 
        F: Fn(&HashMap<String, Vec<DisplayMessage>>) + Send + Sync + 'static
    {
        self.update_callback = Some(Box::new(callback));
    }
    
    /// Add a message to the batch with type filtering
    pub fn add_message(&mut self, tab_id: String, message: DisplayMessage) -> bool {
        // Use MessageType for priority filtering - only add if message type is important
        if self.should_include_message_type(&message.message_type) {
            self.pending_messages
                .entry(tab_id)
                .or_insert_with(Vec::new)
                .push(message);
        }
        
        self.should_flush()
    }
    
    /// Determine if message type should be included in batching
    fn should_include_message_type(&self, msg_type: &MessageType) -> bool {
        match msg_type {
            MessageType::Message | MessageType::Action | MessageType::Notice => true,
            MessageType::Join | MessageType::Part | MessageType::Quit => true, 
            MessageType::System => false, // Skip system messages for better performance
            _ => true, // Include other types by default
        }
    }
    
    /// Add multiple messages at once
    pub fn add_messages(&mut self, tab_id: String, messages: Vec<DisplayMessage>) -> bool {
        self.pending_messages
            .entry(tab_id)
            .or_insert_with(Vec::new)
            .extend(messages);
        
        self.should_flush()
    }
    
    /// Check if batch should be flushed
    pub fn should_flush(&self) -> bool {
        let total_messages: usize = self.pending_messages.values().map(|v| v.len()).sum();
        
        total_messages >= self.batch_size_limit || 
        self.last_update.elapsed() >= self.time_limit
    }
    
    /// Force flush all pending messages
    pub fn flush(&mut self) -> HashMap<String, Vec<DisplayMessage>> {
        if self.pending_messages.is_empty() {
            return HashMap::new();
        }
        
        let messages = std::mem::take(&mut self.pending_messages);
        self.last_update = Instant::now();
        
        // Call update callback if set
        if let Some(ref callback) = self.update_callback {
            callback(&messages);
        }
        
        messages
    }
    
    /// Get current batch statistics
    pub fn stats(&self) -> BatchStats {
        let total_messages: usize = self.pending_messages.values().map(|v| v.len()).sum();
        let tabs_with_messages = self.pending_messages.len();
        
        BatchStats {
            pending_messages: total_messages,
            pending_tabs: tabs_with_messages,
            time_since_last_flush: self.last_update.elapsed(),
            batch_size_limit: self.batch_size_limit,
            time_limit: self.time_limit,
        }
    }
    
    /// Clear all pending messages without flushing
    pub fn clear(&mut self) {
        self.pending_messages.clear();
        self.last_update = Instant::now();
    }
    
    /// Start automatic interval-based flushing for optimal performance
    pub async fn start_auto_flush(&mut self, flush_interval: Duration) {
        let mut timer = interval(flush_interval);
        
        loop {
            timer.tick().await;
            if self.should_flush() {
                let flushed = self.flush();
                if !flushed.is_empty() {
                    if let Some(ref callback) = self.update_callback {
                        callback(&flushed);
                    }
                }
            }
        }
    }
}

/// Batch statistics
#[derive(Debug, Clone)]
pub struct BatchStats {
    pub pending_messages: usize,
    pub pending_tabs: usize,
    pub time_since_last_flush: Duration,
    pub batch_size_limit: usize,
    pub time_limit: Duration,
}

/// Dirty region tracking for efficient rendering
pub struct DirtyRegionTracker {
    dirty_regions: HashMap<String, DirtyRegion>, // region_id -> region
    frame_dirty: bool,
    last_frame_time: Instant,
}

#[derive(Debug, Clone)]
pub struct DirtyRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub priority: DirtyPriority,
    pub last_modified: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DirtyPriority {
    Low,    // Background updates
    Normal, // Regular content updates
    High,   // User interactions
    Critical, // Error states, highlights
}

impl DirtyRegionTracker {
    pub fn new() -> Self {
        Self {
            dirty_regions: HashMap::new(),
            frame_dirty: false,
            last_frame_time: Instant::now(),
        }
    }
    
    /// Mark a region as dirty
    pub fn mark_dirty(&mut self, id: String, x: f32, y: f32, width: f32, height: f32, priority: DirtyPriority) {
        let region = DirtyRegion {
            x,
            y,
            width,
            height,
            priority,
            last_modified: Instant::now(),
        };
        
        self.dirty_regions.insert(id, region);
        self.frame_dirty = true;
    }
    
    /// Mark entire frame as dirty
    pub fn mark_frame_dirty(&mut self) {
        self.frame_dirty = true;
    }
    
    /// Check if frame needs redrawing
    pub fn needs_redraw(&self) -> bool {
        self.frame_dirty || !self.dirty_regions.is_empty()
    }
    
    /// Get dirty regions sorted by priority
    pub fn get_dirty_regions(&self) -> Vec<(&String, &DirtyRegion)> {
        let mut regions: Vec<_> = self.dirty_regions.iter().collect();
        regions.sort_by(|(_, a), (_, b)| b.priority.cmp(&a.priority));
        regions
    }
    
    /// Clear dirty regions after rendering
    pub fn clear_dirty(&mut self) {
        self.dirty_regions.clear();
        self.frame_dirty = false;
        self.last_frame_time = Instant::now();
    }
    
    /// Get regions older than threshold
    pub fn get_stale_regions(&self, threshold: Duration) -> Vec<String> {
        let now = Instant::now();
        self.dirty_regions
            .iter()
            .filter(|(_, region)| now.duration_since(region.last_modified) > threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Remove stale regions
    pub fn cleanup_stale_regions(&mut self, threshold: Duration) {
        let stale_ids = self.get_stale_regions(threshold);
        for id in stale_ids {
            self.dirty_regions.remove(&id);
        }
    }
}

/// Frame rate limiter for smooth animations
pub struct FrameRateLimiter {
    target_fps: u32,
    frame_duration: Duration,
    last_frame: Instant,
    frame_count: u64,
    start_time: Instant,
}

impl FrameRateLimiter {
    /// Create a new frame rate limiter
    pub fn new(target_fps: u32) -> Self {
        let frame_duration = Duration::from_nanos(1_000_000_000 / target_fps as u64);
        let now = Instant::now();
        
        Self {
            target_fps,
            frame_duration,
            last_frame: now,
            frame_count: 0,
            start_time: now,
        }
    }
    
    /// Wait until next frame should be rendered
    pub async fn wait_for_next_frame(&mut self) {
        let elapsed = self.last_frame.elapsed();
        
        if elapsed < self.frame_duration {
            let sleep_duration = self.frame_duration - elapsed;
            tokio::time::sleep(sleep_duration).await;
        }
        
        self.last_frame = Instant::now();
        self.frame_count += 1;
    }
    
    /// Check if it's time for next frame
    pub fn should_render_frame(&self) -> bool {
        self.last_frame.elapsed() >= self.frame_duration
    }
    
    /// Get current FPS
    pub fn current_fps(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        if elapsed > 0.0 {
            self.frame_count as f32 / elapsed
        } else {
            0.0
        }
    }
    
    /// Set new target FPS
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
        self.frame_duration = Duration::from_nanos(1_000_000_000 / fps as u64);
    }
    
    /// Get frame timing statistics
    pub fn stats(&self) -> FrameStats {
        FrameStats {
            target_fps: self.target_fps,
            current_fps: self.current_fps(),
            frame_count: self.frame_count,
            runtime: self.start_time.elapsed(),
            frame_duration: self.frame_duration,
        }
    }
}

/// Frame timing statistics
#[derive(Debug, Clone)]
pub struct FrameStats {
    pub target_fps: u32,
    pub current_fps: f32,
    pub frame_count: u64,
    pub runtime: Duration,
    pub frame_duration: Duration,
}

/// Performance monitor for overall application performance
pub struct PerformanceMonitor {
    message_batcher: MessageBatcher,
    dirty_tracker: DirtyRegionTracker,
    frame_limiter: FrameRateLimiter,
    memory_usage: MemoryTracker,
    cpu_usage: CpuTracker,
}

impl PerformanceMonitor {
    pub fn new(target_fps: u32) -> Self {
        Self {
            message_batcher: MessageBatcher::new(),
            dirty_tracker: DirtyRegionTracker::new(),
            frame_limiter: FrameRateLimiter::new(target_fps),
            memory_usage: MemoryTracker::new(),
            cpu_usage: CpuTracker::new(),
        }
    }
    
    /// Get comprehensive performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            batch_stats: self.message_batcher.stats(),
            frame_stats: self.frame_limiter.stats(),
            memory_stats: self.memory_usage.stats(),
            cpu_stats: self.cpu_usage.stats(),
            dirty_regions_count: self.dirty_tracker.dirty_regions.len(),
            needs_redraw: self.dirty_tracker.needs_redraw(),
        }
    }
    
    /// Update performance metrics
    pub fn update(&mut self) {
        self.memory_usage.update();
        self.cpu_usage.update();
        self.dirty_tracker.cleanup_stale_regions(Duration::from_secs(5));
    }
    
    /// Get message batcher
    pub fn message_batcher(&mut self) -> &mut MessageBatcher {
        &mut self.message_batcher
    }
    
    /// Get dirty region tracker
    pub fn dirty_tracker(&mut self) -> &mut DirtyRegionTracker {
        &mut self.dirty_tracker
    }
    
    /// Get frame rate limiter
    pub fn frame_limiter(&mut self) -> &mut FrameRateLimiter {
        &mut self.frame_limiter
    }
}

/// Memory usage tracker
pub struct MemoryTracker {
    samples: VecDeque<MemorySample>,
    max_samples: usize,
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub heap_usage: usize,
    pub allocated: usize,
    pub resident: usize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            max_samples: 100, // Keep last 100 samples
        }
    }
    
    pub fn update(&mut self) {
        // This would use a real memory profiling library in production
        let sample = MemorySample {
            timestamp: Instant::now(),
            heap_usage: 0, // Placeholder
            allocated: 0,  // Placeholder
            resident: 0,   // Placeholder
        };
        
        self.samples.push_back(sample);
        if self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }
    
    pub fn stats(&self) -> MemoryStats {
        if let Some(latest) = self.samples.back() {
            MemoryStats {
                current_heap: latest.heap_usage,
                current_allocated: latest.allocated,
                current_resident: latest.resident,
                samples_count: self.samples.len(),
            }
        } else {
            MemoryStats::default()
        }
    }
}

/// CPU usage tracker
pub struct CpuTracker {
    samples: VecDeque<CpuSample>,
    max_samples: usize,
}

#[derive(Debug, Clone)]
pub struct CpuSample {
    pub timestamp: Instant,
    pub cpu_percent: f32,
}

impl CpuTracker {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            max_samples: 60, // One minute of samples
        }
    }
    
    pub fn update(&mut self) {
        // This would use a real CPU monitoring library in production
        let sample = CpuSample {
            timestamp: Instant::now(),
            cpu_percent: 0.0, // Placeholder
        };
        
        self.samples.push_back(sample);
        if self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }
    
    pub fn stats(&self) -> CpuStats {
        if let Some(latest) = self.samples.back() {
            let average = if !self.samples.is_empty() {
                self.samples.iter().map(|s| s.cpu_percent).sum::<f32>() / self.samples.len() as f32
            } else {
                0.0
            };
            
            CpuStats {
                current_cpu: latest.cpu_percent,
                average_cpu: average,
                samples_count: self.samples.len(),
            }
        } else {
            CpuStats::default()
        }
    }
}

/// Performance statistics summary
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub batch_stats: BatchStats,
    pub frame_stats: FrameStats,
    pub memory_stats: MemoryStats,
    pub cpu_stats: CpuStats,
    pub dirty_regions_count: usize,
    pub needs_redraw: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub current_heap: usize,
    pub current_allocated: usize,
    pub current_resident: usize,
    pub samples_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CpuStats {
    pub current_cpu: f32,
    pub average_cpu: f32,
    pub samples_count: usize,
}

impl Default for MessageBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DirtyRegionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CpuTracker {
    fn default() -> Self {
        Self::new()
    }
}