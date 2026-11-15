//! Task definition
//!
//! Defines schedulable tasks

use crate::policy::SchedulerPolicy;
use crate::affinity::CpuSet;

/// Task ID type
pub type TaskId = u64;

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Runnable,
    Sleeping,
    Stopped,
    Zombie,
    Dead,
}

/// Task
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub pid: u32,
    pub tid: u32,
    pub priority: i32, // nice value (-20 to 19)
    pub rt_priority: Option<i32>, // real-time priority (1-99)
    pub vruntime: u64,
    pub deadline: u64,
    pub runtime: u64,
    pub period: u64,
    pub policy: SchedulerPolicy,
    pub affinity: CpuSet,
    pub state: TaskState,
    pub start_time: u64,
    pub last_run: u64,
    pub exec_start: u64,
    pub sum_exec_runtime: u64,
}

impl Task {
    /// Create new task
    pub fn new(pid: u32, policy: SchedulerPolicy, priority: i32) -> Self {
        Self {
            id: 0,
            pid,
            tid: pid,
            priority,
            rt_priority: None,
            vruntime: 0,
            deadline: 0,
            runtime: 0,
            period: 0,
            policy,
            affinity: CpuSet::all(),
            state: TaskState::Runnable,
            start_time: 0,
            last_run: 0,
            exec_start: 0,
            sum_exec_runtime: 0,
        }
    }
    
    /// Get time slice for task
    pub fn get_time_slice(&self) -> u64 {
        match self.policy {
            SchedulerPolicy::Normal => {
                // CFS time slice calculation
                // Base time slice adjusted by nice value
                let base = 6_000_000; // 6ms in nanoseconds
                let nice_factor = (self.priority + 20) as f64 / 40.0;
                (base as f64 * nice_factor) as u64
            }
            SchedulerPolicy::RealTimeFifo | SchedulerPolicy::RealTimeRoundRobin => {
                // Real-time time slice
                10_000_000 // 10ms
            }
            SchedulerPolicy::Deadline => {
                // Deadline runtime
                self.runtime
            }
            SchedulerPolicy::Idle => {
                // Idle tasks get minimal time
                1_000_000 // 1ms
            }
        }
    }
    
    /// Check if task is real-time
    pub fn is_realtime(&self) -> bool {
        matches!(self.policy, SchedulerPolicy::RealTimeFifo | SchedulerPolicy::RealTimeRoundRobin | SchedulerPolicy::Deadline)
    }
    
    /// Check if task is deadline
    pub fn is_deadline(&self) -> bool {
        self.policy == SchedulerPolicy::Deadline
    }
    
    /// Update task statistics
    pub fn update_stats(&mut self, runtime_ns: u64) {
        self.sum_exec_runtime += runtime_ns;
        self.last_run = runtime_ns;
        self.exec_start = runtime_ns;
    }
}
