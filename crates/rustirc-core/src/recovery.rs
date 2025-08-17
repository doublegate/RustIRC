//! Connection recovery and error handling
//!
//! This module provides comprehensive error recovery capabilities:
//! - Automatic reconnection with exponential backoff
//! - Connection health monitoring
//! - State recovery after reconnection
//! - Error classification and handling strategies
//! - Circuit breaker pattern for failing connections

use crate::connection::{ConnectionConfig, ConnectionState, IrcConnection};
use crate::error::{Error, Result};
use crate::events::{Event, EventBus};
use crate::state::{StateManager, ServerState};
use rustirc_protocol::Command;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

/// Recovery strategy for different types of errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Immediately retry connection
    ImmediateRetry,
    /// Retry with exponential backoff
    ExponentialBackoff,
    /// Manual intervention required
    Manual,
    /// Permanent failure, don't retry
    Permanent,
}

/// Error classification for recovery decisions
#[derive(Debug, Clone)]
pub enum ErrorType {
    /// Network connection failed
    NetworkError,
    /// DNS resolution failed
    DnsError,
    /// TLS/SSL handshake failed
    TlsError,
    /// Authentication failed
    AuthError,
    /// Server rejected connection
    ServerError,
    /// Timeout occurred
    Timeout,
    /// Rate limiting
    RateLimit,
    /// Protocol violation
    ProtocolError,
    /// Unknown error
    Unknown,
}

impl ErrorType {
    /// Get recovery strategy for this error type
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            ErrorType::NetworkError => RecoveryStrategy::ExponentialBackoff,
            ErrorType::DnsError => RecoveryStrategy::ExponentialBackoff,
            ErrorType::TlsError => RecoveryStrategy::Manual,
            ErrorType::AuthError => RecoveryStrategy::Manual,
            ErrorType::ServerError => RecoveryStrategy::ExponentialBackoff,
            ErrorType::Timeout => RecoveryStrategy::ExponentialBackoff,
            ErrorType::RateLimit => RecoveryStrategy::ExponentialBackoff,
            ErrorType::ProtocolError => RecoveryStrategy::Manual,
            ErrorType::Unknown => RecoveryStrategy::ExponentialBackoff,
        }
    }

    /// Classify error from error message
    pub fn from_error(error: &Error) -> Self {
        match error {
            Error::ConnectionFailed(_) => ErrorType::NetworkError,
            Error::ConnectionTimeout => ErrorType::Timeout,
            Error::InvalidAddress(_) => ErrorType::DnsError,
            Error::InvalidTlsName(_) => ErrorType::TlsError,
            Error::TlsError(_) => ErrorType::TlsError,
            Error::Protocol(_) => ErrorType::ProtocolError,
            _ => ErrorType::Unknown,
        }
    }
}

/// Reconnection configuration
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    pub enabled: bool,
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 10,
            initial_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(300), // 5 minutes
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq)]
enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing, not allowing connections
    HalfOpen,   // Testing if service is back
}

/// Circuit breaker for connection failures
#[derive(Debug)]
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    timeout: Duration,
    last_failure: Option<Instant>,
    success_threshold: u32, // Successes needed in half-open to close
    half_open_successes: u32,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout: Duration, success_threshold: u32) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            timeout,
            last_failure: None,
            success_threshold,
            half_open_successes: 0,
        }
    }

    /// Check if we should allow a connection attempt
    fn should_allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() > self.timeout {
                        self.state = CircuitState::HalfOpen;
                        self.half_open_successes = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.half_open_successes += 1;
                if self.half_open_successes >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Open => {
                // Should not happen
            }
        }
    }

    /// Record a failed operation
    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.half_open_successes = 0;
            }
            CircuitState::Open => {
                // Already open
            }
        }
    }
}

/// Recovery manager for a single connection
#[derive(Debug)]
pub struct ConnectionRecovery {
    connection_id: String,
    config: ReconnectConfig,
    circuit_breaker: CircuitBreaker,
    current_attempt: u32,
    last_attempt: Option<Instant>,
    state_before_disconnect: Option<ServerState>,
}

impl ConnectionRecovery {
    pub fn new(connection_id: String, config: ReconnectConfig) -> Self {
        Self {
            connection_id,
            config,
            circuit_breaker: CircuitBreaker::new(5, Duration::from_secs(60), 2),
            current_attempt: 0,
            last_attempt: None,
            state_before_disconnect: None,
        }
    }

    /// Check if we should attempt reconnection
    pub fn should_reconnect(&mut self) -> bool {
        if !self.config.enabled {
            return false;
        }

        if self.current_attempt >= self.config.max_attempts {
            return false;
        }

        if !self.circuit_breaker.should_allow_request() {
            return false;
        }

        // Check if enough time has passed since last attempt
        if let Some(last_attempt) = self.last_attempt {
            let delay = self.calculate_delay();
            if last_attempt.elapsed() < delay {
                return false;
            }
        }

        true
    }

    /// Calculate delay for next reconnection attempt
    fn calculate_delay(&self) -> Duration {
        let mut delay = self.config.initial_delay.as_millis() as f64
            * self.config.backoff_multiplier.powi(self.current_attempt as i32);

        delay = delay.min(self.config.max_delay.as_millis() as f64);

        // Add jitter if enabled
        if self.config.jitter {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.8..1.2);
            delay *= jitter_factor;
        }

        Duration::from_millis(delay as u64)
    }

    /// Record connection attempt
    pub fn record_attempt(&mut self) {
        self.current_attempt += 1;
        self.last_attempt = Some(Instant::now());
    }

    /// Record successful connection
    pub fn record_success(&mut self) {
        self.current_attempt = 0;
        self.circuit_breaker.record_success();
    }

    /// Record failed connection
    pub fn record_failure(&mut self, error: &Error) {
        let error_type = ErrorType::from_error(error);
        self.circuit_breaker.record_failure();

        warn!(
            "Connection {} failed (attempt {}): {} (type: {:?})",
            self.connection_id, self.current_attempt, error, error_type
        );
    }

    /// Save state before disconnection for recovery
    pub fn save_state(&mut self, state: ServerState) {
        self.state_before_disconnect = Some(state);
    }

    /// Get saved state for recovery
    pub fn get_saved_state(&self) -> Option<&ServerState> {
        self.state_before_disconnect.as_ref()
    }
}

/// Global recovery manager
pub struct RecoveryManager {
    connections: Arc<RwLock<HashMap<String, ConnectionRecovery>>>,
    state_manager: Arc<StateManager>,
    event_bus: Arc<EventBus>,
    recovery_tx: mpsc::UnboundedSender<RecoveryTask>,
}

/// Recovery task
#[derive(Debug)]
pub enum RecoveryTask {
    ScheduleReconnect { connection_id: String },
    RestoreState { connection_id: String },
    HealthCheck { connection_id: String },
}

impl RecoveryManager {
    pub fn new(
        state_manager: Arc<StateManager>,
        event_bus: Arc<EventBus>,
    ) -> (Self, mpsc::UnboundedReceiver<RecoveryTask>) {
        let (recovery_tx, recovery_rx) = mpsc::unbounded_channel();

        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            state_manager,
            event_bus,
            recovery_tx,
        };

        (manager, recovery_rx)
    }

    /// Register a connection for recovery
    pub async fn register_connection(&self, connection_id: String, config: ReconnectConfig) {
        let recovery = ConnectionRecovery::new(connection_id.clone(), config);
        self.connections.write().await.insert(connection_id, recovery);
    }

    /// Handle connection failure
    pub async fn handle_connection_failure(
        &self,
        connection_id: String,
        error: Error,
    ) -> Result<()> {
        // Save current state before handling failure
        if let Some(server_state) = self.state_manager.get_server_state(&connection_id).await {
            let mut connections = self.connections.write().await;
            if let Some(recovery) = connections.get_mut(&connection_id) {
                recovery.save_state(server_state);
                recovery.record_failure(&error);

                if recovery.should_reconnect() {
                    info!("Scheduling reconnection for {}", connection_id);
                    self.recovery_tx
                        .send(RecoveryTask::ScheduleReconnect { connection_id })
                        .map_err(|_| Error::ChannelSend)?;
                } else {
                    warn!(
                        "Connection {} will not be reconnected (max attempts or circuit breaker)",
                        connection_id
                    );
                }
            }
        }

        Ok(())
    }

    /// Handle successful connection
    pub async fn handle_connection_success(&self, connection_id: String) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(recovery) = connections.get_mut(&connection_id) {
            recovery.record_success();
            
            // Schedule state restoration
            self.recovery_tx
                .send(RecoveryTask::RestoreState { connection_id })
                .map_err(|_| Error::ChannelSend)?;
        }

        Ok(())
    }

    /// Restore connection state after reconnection
    pub async fn restore_connection_state(&self, connection_id: String) -> Result<()> {
        let connections = self.connections.read().await;
        if let Some(recovery) = connections.get(&connection_id) {
            if let Some(saved_state) = recovery.get_saved_state() {
                info!("Restoring state for connection {}", connection_id);

                // Rejoin channels
                for (channel_name, channel_state) in &saved_state.channels {
                    if channel_state.joined {
                        let join_cmd = Command::Join {
                            channels: vec![channel_name.clone()],
                            keys: vec![],
                        };

                        // Send through event system
                        let event = Event::MessageSent {
                            connection_id: connection_id.clone(),
                            message: join_cmd.to_message(),
                        };
                        self.event_bus.emit(event).await;
                    }
                }

                // Restore nickname if different
                if !saved_state.nickname.is_empty() {
                    let nick_cmd = Command::Nick {
                        nickname: saved_state.nickname.clone(),
                    };

                    let event = Event::MessageSent {
                        connection_id: connection_id.clone(),
                        message: nick_cmd.to_message(),
                    };
                    self.event_bus.emit(event).await;
                }

                info!("State restoration completed for {}", connection_id);
            }
        }

        Ok(())
    }

    /// Start health monitoring for all connections
    pub async fn start_health_monitoring(&self) -> Result<()> {
        let connections = self.connections.clone();
        let recovery_tx = self.recovery_tx.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                let connection_ids: Vec<String> = connections
                    .read()
                    .await
                    .keys()
                    .cloned()
                    .collect();

                for connection_id in connection_ids {
                    if recovery_tx
                        .send(RecoveryTask::HealthCheck { connection_id })
                        .is_err()
                    {
                        break; // Channel closed
                    }
                }
            }
        });

        Ok(())
    }

    /// Perform health check for a connection
    pub async fn health_check(&self, connection_id: String) -> Result<()> {
        // Check if connection is responsive
        // This would typically involve sending a PING and waiting for PONG
        debug!("Health check for connection {}", connection_id);

        // For now, this is a placeholder
        // In a real implementation, this would:
        // 1. Check last activity time
        // 2. Send PING if needed
        // 3. Mark connection as failed if no response

        Ok(())
    }

    /// Get recovery statistics
    pub async fn get_recovery_stats(&self) -> HashMap<String, RecoveryStats> {
        let connections = self.connections.read().await;
        let mut stats = HashMap::new();

        for (connection_id, recovery) in connections.iter() {
            stats.insert(
                connection_id.clone(),
                RecoveryStats {
                    attempt_count: recovery.current_attempt,
                    last_attempt: recovery.last_attempt,
                    circuit_state: recovery.circuit_breaker.state.clone(),
                    has_saved_state: recovery.state_before_disconnect.is_some(),
                },
            );
        }

        stats
    }
}

/// Recovery statistics
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub attempt_count: u32,
    pub last_attempt: Option<Instant>,
    pub circuit_state: CircuitState,
    pub has_saved_state: bool,
}

/// Recovery task processor
pub async fn process_recovery_tasks(
    mut recovery_rx: mpsc::UnboundedReceiver<RecoveryTask>,
    recovery_manager: Arc<RecoveryManager>,
) {
    while let Some(task) = recovery_rx.recv().await {
        match task {
            RecoveryTask::ScheduleReconnect { connection_id } => {
                // Calculate delay and schedule reconnection
                if let Some(recovery) = recovery_manager
                    .connections
                    .read()
                    .await
                    .get(&connection_id)
                {
                    let delay = recovery.calculate_delay();
                    let connection_id_clone = connection_id.clone();
                    let recovery_tx = recovery_manager.recovery_tx.clone();

                    tokio::spawn(async move {
                        sleep(delay).await;
                        // Signal that it's time to attempt reconnection
                        // This would be handled by the connection manager
                        debug!("Reconnection timer expired for {}", connection_id_clone);
                    });
                }
            }
            RecoveryTask::RestoreState { connection_id } => {
                if let Err(e) = recovery_manager.restore_connection_state(connection_id).await {
                    error!("Failed to restore connection state: {}", e);
                }
            }
            RecoveryTask::HealthCheck { connection_id } => {
                if let Err(e) = recovery_manager.health_check(connection_id).await {
                    error!("Health check failed: {}", e);
                }
            }
        }
    }
}