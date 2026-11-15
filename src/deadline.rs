//! Deadline scheduling implementation
//!
//! Earliest Deadline First (EDF) scheduling

use crate::task::{Task, TaskId};

/// Deadline scheduler
pub struct DeadlineScheduler {
    pub tasks: Vec<(u64, TaskId)>, // (deadline, task_id)
    pub current_task: Option<TaskId>,
    pub runtime_budget: u64,
}

impl DeadlineScheduler {
    /// Create new deadline scheduler
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_task: None,
            runtime_budget: 0,
        }
    }
    
    /// Enqueue task
    pub fn enqueue_task(&mut self, task: &Task) {
        if task.is_deadline() {
            self.tasks.push((task.deadline, task.id));
            // Sort by deadline (earliest first)
            self.tasks.sort_by_key(|&(deadline, _)| deadline);
        }
    }
    
    /// Dequeue task
    pub fn dequeue_task(&mut self, task_id: TaskId) -> bool {
        if let Some(pos) = self.tasks.iter().position(|&(_, id)| id == task_id) {
            self.tasks.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Pick next task (earliest deadline)
    pub fn pick_next_task(&mut self) -> Option<TaskId> {
        if !self.tasks.is_empty() {
            let (_, task_id) = self.tasks[0];
            self.current_task = Some(task_id);
            return Some(task_id);
        }
        None
    }
    
    /// Task tick (check runtime budget)
    pub fn task_tick(&mut self, task: &mut Task, runtime_ns: u64) {
        // Update runtime budget
        self.runtime_budget = self.runtime_budget.saturating_sub(runtime_ns);
        
        // Check if deadline task has exceeded runtime
        if task.is_deadline() && runtime_ns >= task.runtime {
            // Task exhausted its runtime budget
            // Would trigger admission control or throttling
        }
        
        // Update task statistics
        task.update_stats(runtime_ns);
    }
    
    /// Admission control: check if task can be admitted
    pub fn can_admit(&self, task: &Task) -> bool {
        if !task.is_deadline() {
            return true;
        }
        
        // Check if adding this task would overload the system
        // Simplified: check if total utilization < 100%
        let total_runtime: u64 = self.tasks.iter().map(|_| 0).sum(); // Simplified
        let new_utilization = (total_runtime + task.runtime) as f64 / task.period as f64;
        
        new_utilization <= 1.0
    }
    
    /// Get current utilization
    pub fn utilization(&self) -> f64 {
        // Simplified utilization calculation
        let total_runtime: u64 = self.tasks.iter().map(|_| 0).sum();
        let total_period: u64 = self.tasks.iter().map(|_| 0).sum();
        
        if total_period > 0 {
            total_runtime as f64 / total_period as f64
        } else {
            0.0
        }
    }
}
