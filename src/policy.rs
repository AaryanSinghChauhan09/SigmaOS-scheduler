//! Scheduler policies
//!
//! Defines scheduling policies

use serde::{Serialize, Deserialize};

/// Scheduler policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerPolicy {
    /// Normal tasks (CFS/EEVDF)
    Normal,
    /// Real-time FIFO
    RealTimeFifo,
    /// Real-time Round-Robin
    RealTimeRoundRobin,
    /// Deadline scheduling
    Deadline,
    /// Idle tasks
    Idle,
}

impl SchedulerPolicy {
    /// Get policy priority (higher is more important)
    pub fn priority(&self) -> i32 {
        match self {
            SchedulerPolicy::Deadline => 4,
            SchedulerPolicy::RealTimeFifo => 3,
            SchedulerPolicy::RealTimeRoundRobin => 2,
            SchedulerPolicy::Normal => 1,
            SchedulerPolicy::Idle => 0,
        }
    }
    
    /// Check if policy is real-time
    pub fn is_realtime(&self) -> bool {
        matches!(self, SchedulerPolicy::RealTimeFifo | SchedulerPolicy::RealTimeRoundRobin | SchedulerPolicy::Deadline)
    }
    
    /// Check if policy is deadline
    pub fn is_deadline(&self) -> bool {
        self == &SchedulerPolicy::Deadline
    }
    
    /// Get time slice for policy
    pub fn get_time_slice(&self) -> u64 {
        match self {
            SchedulerPolicy::Normal => 6_000_000, // 6ms
            SchedulerPolicy::RealTimeFifo => 10_000_000, // 10ms
            SchedulerPolicy::RealTimeRoundRobin => 10_000_000, // 10ms
            SchedulerPolicy::Deadline => 0, // Variable
            SchedulerPolicy::Idle => 1_000_000, // 1ms
        }
    }
}
