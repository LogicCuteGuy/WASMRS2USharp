//! Performance monitoring system for UdonSharp compilation pipeline

use crate::metrics::{CompilationMetrics, PerformanceMetrics, MemoryMetrics};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Main performance monitor for tracking compilation metrics
#[derive(Debug)]
pub struct UdonPerformanceMonitor {
    sessions: Arc<Mutex<HashMap<String, MonitoringSession>>>,
    global_metrics: Arc<Mutex<GlobalMetrics>>,
}

/// A monitoring session for tracking a complete compilation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSession {
    pub id: String,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub steps: Vec<CompilationStep>,
    pub metrics: CompilationMetrics,
    pub memory_usage: Vec<MemorySnapshot>,
    pub status: SessionStatus,
}

/// Individual compilation step within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStep {
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub memory_before: u64,
    pub memory_after: u64,
    pub status: StepStatus,
    pub error_message: Option<String>,
    pub sub_steps: Vec<CompilationStep>,
}

/// Memory usage snapshot at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: DateTime<Utc>,
    pub heap_used: u64,
    pub heap_total: u64,
    pub stack_used: u64,
    pub step_name: String,
}

/// Status of a monitoring session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
    Cancelled,
}

/// Status of a compilation step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Global metrics across all sessions
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    pub total_sessions: u64,
    pub successful_sessions: u64,
    pub failed_sessions: u64,
    pub average_compilation_time: Duration,
    pub total_compilation_time: Duration,
    pub peak_memory_usage: u64,
    pub average_memory_usage: u64,
}

impl UdonPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Result<Self> {
        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            global_metrics: Arc::new(Mutex::new(GlobalMetrics::default())),
        })
    }

    /// Start a new monitoring session
    pub fn start_session(&self, name: &str) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let session = MonitoringSession {
            id: session_id.clone(),
            name: name.to_string(),
            start_time: Utc::now(),
            end_time: None,
            steps: Vec::new(),
            metrics: CompilationMetrics::default(),
            memory_usage: Vec::new(),
            status: SessionStatus::Active,
        };

        let mut sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        sessions.insert(session_id.clone(), session);

        // Take initial memory snapshot
        self.take_memory_snapshot(&session_id, "session_start")?;

        Ok(session_id)
    }

    /// End a monitoring session
    pub fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.end_time = Some(Utc::now());
            session.status = SessionStatus::Completed;

            // Calculate final metrics
            if let Some(start_time) = session.start_time.timestamp_nanos_opt() {
                if let Some(end_time) = session.end_time.and_then(|t| t.timestamp_nanos_opt()) {
                    let duration = Duration::from_nanos((end_time - start_time) as u64);
                    session.metrics.total_compilation_time = duration;
                }
            }

            // Take final memory snapshot
            self.take_memory_snapshot(session_id, "session_end")?;

            // Update global metrics
            self.update_global_metrics(session)?;

            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Start a compilation step within a session
    pub fn start_step(&self, session_id: &str, step_name: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        
        if let Some(session) = sessions.get_mut(session_id) {
            let step = CompilationStep {
                name: step_name.to_string(),
                start_time: Utc::now(),
                end_time: None,
                duration: None,
                memory_before: self.get_current_memory_usage(),
                memory_after: 0,
                status: StepStatus::Running,
                error_message: None,
                sub_steps: Vec::new(),
            };

            session.steps.push(step);
            self.take_memory_snapshot(session_id, step_name)?;

            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// End a compilation step within a session
    pub fn end_step(&self, session_id: &str, step_name: &str, success: bool, error: Option<String>) -> Result<()> {
        let mut sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(step) = session.steps.iter_mut().find(|s| s.name == step_name && s.end_time.is_none()) {
                step.end_time = Some(Utc::now());
                step.memory_after = self.get_current_memory_usage();
                step.status = if success { StepStatus::Completed } else { StepStatus::Failed };
                step.error_message = error;

                // Calculate duration
                if let Some(start_nanos) = step.start_time.timestamp_nanos_opt() {
                    if let Some(end_nanos) = step.end_time.and_then(|t| t.timestamp_nanos_opt()) {
                        step.duration = Some(Duration::from_nanos((end_nanos - start_nanos) as u64));
                    }
                }

                self.take_memory_snapshot(session_id, &format!("{}_end", step_name))?;

                Ok(())
            } else {
                Err(anyhow!("Step not found or already completed: {}", step_name))
            }
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Get metrics for a specific session
    pub fn get_session_metrics(&self, session_id: &str) -> Result<PerformanceMetrics> {
        let sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        
        if let Some(session) = sessions.get(session_id) {
            let compilation_metrics = &session.metrics;
            let memory_metrics = self.calculate_memory_metrics(&session.memory_usage);

            Ok(PerformanceMetrics {
                compilation: compilation_metrics.clone(),
                memory: memory_metrics,
                session_duration: session.end_time
                    .map(|end| {
                        if let (Some(start_nanos), Some(end_nanos)) = 
                            (session.start_time.timestamp_nanos_opt(), end.timestamp_nanos_opt()) {
                            Duration::from_nanos((end_nanos - start_nanos) as u64)
                        } else {
                            Duration::ZERO
                        }
                    })
                    .unwrap_or(Duration::ZERO),
                step_count: session.steps.len(),
                failed_steps: session.steps.iter().filter(|s| s.status == StepStatus::Failed).count(),
            })
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Get global performance metrics
    pub fn get_global_metrics(&self) -> Result<GlobalMetrics> {
        let metrics = self.global_metrics.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        Ok((*metrics).clone())
    }

    /// Take a memory snapshot at the current time
    fn take_memory_snapshot(&self, session_id: &str, step_name: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        
        if let Some(session) = sessions.get_mut(session_id) {
            let snapshot = MemorySnapshot {
                timestamp: Utc::now(),
                heap_used: self.get_current_memory_usage(),
                heap_total: self.get_total_memory(),
                stack_used: self.get_stack_usage(),
                step_name: step_name.to_string(),
            };

            session.memory_usage.push(snapshot);
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Get current memory usage (simplified implementation)
    fn get_current_memory_usage(&self) -> u64 {
        // In a real implementation, this would use system APIs to get actual memory usage
        // For now, we'll use a placeholder that estimates based on process info
        std::process::id() as u64 * 1024 // Placeholder
    }

    /// Get total available memory
    fn get_total_memory(&self) -> u64 {
        // Placeholder implementation
        8 * 1024 * 1024 * 1024 // 8GB placeholder
    }

    /// Get current stack usage
    fn get_stack_usage(&self) -> u64 {
        // Placeholder implementation
        1024 * 1024 // 1MB placeholder
    }

    /// Calculate memory metrics from snapshots
    fn calculate_memory_metrics(&self, snapshots: &[MemorySnapshot]) -> MemoryMetrics {
        if snapshots.is_empty() {
            return MemoryMetrics::default();
        }

        let peak_usage = snapshots.iter().map(|s| s.heap_used).max().unwrap_or(0);
        let average_usage = snapshots.iter().map(|s| s.heap_used).sum::<u64>() / snapshots.len() as u64;
        let initial_usage = snapshots.first().map(|s| s.heap_used).unwrap_or(0);
        let final_usage = snapshots.last().map(|s| s.heap_used).unwrap_or(0);

        MemoryMetrics {
            peak_usage,
            average_usage,
            initial_usage,
            final_usage,
            memory_growth: final_usage.saturating_sub(initial_usage),
            gc_collections: 0, // Placeholder
            allocation_rate: 0.0, // Placeholder
        }
    }

    /// Update global metrics with session data
    fn update_global_metrics(&self, session: &MonitoringSession) -> Result<()> {
        let mut global = self.global_metrics.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        
        global.total_sessions += 1;
        
        match session.status {
            SessionStatus::Completed => global.successful_sessions += 1,
            SessionStatus::Failed => global.failed_sessions += 1,
            _ => {}
        }

        // Update timing metrics
        if let Some(duration) = session.end_time.map(|end| {
            if let (Some(start_nanos), Some(end_nanos)) = 
                (session.start_time.timestamp_nanos_opt(), end.timestamp_nanos_opt()) {
                Duration::from_nanos((end_nanos - start_nanos) as u64)
            } else {
                Duration::ZERO
            }
        }) {
            global.total_compilation_time += duration;
            global.average_compilation_time = global.total_compilation_time / global.total_sessions as u32;
        }

        // Update memory metrics
        if let Some(peak_memory) = session.memory_usage.iter().map(|s| s.heap_used).max() {
            if peak_memory > global.peak_memory_usage {
                global.peak_memory_usage = peak_memory;
            }
            
            let session_avg = session.memory_usage.iter().map(|s| s.heap_used).sum::<u64>() / session.memory_usage.len().max(1) as u64;
            global.average_memory_usage = (global.average_memory_usage * (global.total_sessions - 1) + session_avg) / global.total_sessions;
        }

        Ok(())
    }

    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Result<Vec<String>> {
        let sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        Ok(sessions.values()
            .filter(|s| s.status == SessionStatus::Active)
            .map(|s| s.id.clone())
            .collect())
    }

    /// Cancel a session
    pub fn cancel_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Cancelled;
            session.end_time = Some(Utc::now());
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }
}