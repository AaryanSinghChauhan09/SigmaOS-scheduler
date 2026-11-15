//! Real-time scheduling implementation
//!
//! SCHED_FIFO and SCHED_RR policies

use crate::task::{Task, TaskId};
use crate::policy::SchedulerPolicy;

/// Real-time scheduler
pub struct RealtimeScheduler {
    pub fifo_tasks: Vec<TaskId>,
    pub rr_tasks: Vec<TaskId>,
    pub current_task: Option<TaskId>,
    pub rr_time_slice: u64,
}

impl RealtimeScheduler {
    /// Create new realtime scheduler
    pub fn new() -> Self {
        Self {
            fifo_tasks: Vec::new(),
            rr_tasks: Vec::new(),
            current_task: None,
            rr_time_slice: 10_000_000, // 10ms
        }
    }
    
    /// Enqueue task
    pub fn enqueue_task(&mut self, task: &Task) {
        match task.policy {
            SchedulerPolicy::RealTimeFifo => {
                self.fifo_tasks.push(task.id);
            }
            SchedulerPolicy::RealTimeRoundRobin => {
                self.rr_tasks.push(task.id);
            }
            _ => {}
        }
    }
    
    /// Dequeue task
    pub fn dequeue_task(&mut self, task_id: TaskId) -> bool {
        let mut removed = false;
        
        if let Some(pos) = self.fifo_tasks.iter().position(|&id| id == task_id) {
            self.fifo_tasks.remove(pos);
            removed = true;
        }
        
        if let Some(pos) = self.rr_tasks.iter().position(|&id| id == task_id) {
            self.rr_tasks.remove(pos);
            removed = true;
        }
        
        removed
    }
    
    /// Pick next task
    pub fn pick_next_task(&mut self) -> Option<TaskId> {
        // FIFO tasks have highest priority
        if !self.fifo_tasks.is_empty() {
            let task_id = self.fifo_tasks.remove(0);
            self.current_task = Some(task_id);
            return Some(task_id);
        }
        
        // RR tasks with round-robin
        if !self.rr_tasks.is_empty() {
            let task_id = self.rr_tasks.remove(0);
            self.rr_tasks.push(task_id); // Move to end
            self.current_task = Some(task_id);
            return Some(task_id);
        }
        
        None
    }
    
    /// Task tick
    pub fn task_tick(&mut self, task: &mut Task, runtime_ns: u64) {
        // Update task statistics
        task.update_stats(runtime_ns);
        
        // For RR tasks, check if time slice expired
        if task.policy == SchedulerPolicy::RealTimeRoundRobin {
            if runtime_ns >= self.rr_time_slice {
                // Requeue task
                self.enqueue_task(task);
            }
        }
    }
    
    /// Check if should preempt current task
    pub fn check_preempt_curr(&self, task: &Task) -> bool {
        // Real-time tasks always preempt normal tasks
        if task.is_realtime() {
            return true;
        }
        false
    }
}
