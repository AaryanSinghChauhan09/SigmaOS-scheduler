//! Runqueue implementation
//!
//! Per-CPU run queue for scheduling

use crate::task::{Task, TaskId};
use crate::policy::SchedulerPolicy;
use std::collections::BTreeMap;

/// Runqueue
pub struct RunQueue {
    pub cpu_id: usize,
    pub cfs_tasks: BTreeMap<u64, TaskId>, // vruntime -> task_id
    pub rt_fifo_tasks: Vec<TaskId>,
    pub rt_rr_tasks: Vec<TaskId>,
    pub deadline_tasks: Vec<TaskId>,
    pub idle_tasks: Vec<TaskId>,
    pub current_task: Option<TaskId>,
    pub load: LoadInfo,
    pub nr_running: usize,
    pub min_vruntime: u64,
}

/// Load information
#[derive(Debug, Clone)]
pub struct LoadInfo {
    pub runnable: usize,
    pub weight: u64,
    pub avg_load: f32,
}

impl RunQueue {
    /// Create new runqueue
    pub fn new(cpu_id: usize) -> Self {
        Self {
            cpu_id,
            cfs_tasks: BTreeMap::new(),
            rt_fifo_tasks: Vec::new(),
            rt_rr_tasks: Vec::new(),
            deadline_tasks: Vec::new(),
            idle_tasks: Vec::new(),
            current_task: None,
            load: LoadInfo {
                runnable: 0,
                weight: 0,
                avg_load: 0.0,
            },
            nr_running: 0,
            min_vruntime: 0,
        }
    }
    
    /// Enqueue task
    pub fn enqueue(&mut self, task: Task) {
        match task.policy {
            SchedulerPolicy::Normal => {
                self.cfs_tasks.insert(task.vruntime, task.id);
            }
            SchedulerPolicy::RealTimeFifo => {
                self.rt_fifo_tasks.push(task.id);
            }
            SchedulerPolicy::RealTimeRoundRobin => {
                self.rt_rr_tasks.push(task.id);
            }
            SchedulerPolicy::Deadline => {
                self.deadline_tasks.push(task.id);
            }
            SchedulerPolicy::Idle => {
                self.idle_tasks.push(task.id);
            }
        }
        self.nr_running += 1;
        self.update_load();
    }
    
    /// Dequeue task
    pub fn dequeue(&mut self, task_id: TaskId) -> bool {
        // Remove from appropriate queue
        let mut removed = false;
        
        // CFS tasks
        if let Some((_, _)) = self.cfs_tasks.iter().find(|(_, &id)| id == task_id) {
            if let Some(key) = self.cfs_tasks.iter().find(|(_, &id)| id == task_id).map(|(&k, _)| k) {
                self.cfs_tasks.remove(&key);
                removed = true;
            }
        }
        
        // RT FIFO tasks
        if let Some(pos) = self.rt_fifo_tasks.iter().position(|&id| id == task_id) {
            self.rt_fifo_tasks.remove(pos);
            removed = true;
        }
        
        // RT RR tasks
        if let Some(pos) = self.rt_rr_tasks.iter().position(|&id| id == task_id) {
            self.rt_rr_tasks.remove(pos);
            removed = true;
        }
        
        // Deadline tasks
        if let Some(pos) = self.deadline_tasks.iter().position(|&id| id == task_id) {
            self.deadline_tasks.remove(pos);
            removed = true;
        }
        
        // Idle tasks
        if let Some(pos) = self.idle_tasks.iter().position(|&id| id == task_id) {
            self.idle_tasks.remove(pos);
            removed = true;
        }
        
        if removed {
            self.nr_running = self.nr_running.saturating_sub(1);
            self.update_load();
        }
        
        removed
    }
    
    /// Pick next task to run
    pub fn pick_next(&mut self) -> Option<TaskId> {
        // Priority order: Deadline > RT FIFO > RT RR > CFS > Idle
        
        // Deadline tasks (earliest deadline first)
        if !self.deadline_tasks.is_empty() {
            // Sort by deadline
            self.deadline_tasks.sort_by_key(|&id| {
                // Would need task access to get deadline
                // For now, simple approach
                id
            });
            return self.deadline_tasks.first().copied();
        }
        
        // Real-time FIFO tasks (highest priority first)
        if !self.rt_fifo_tasks.is_empty() {
            return self.rt_fifo_tasks.first().copied();
        }
        
        // Real-time RR tasks
        if !self.rt_rr_tasks.is_empty() {
            return self.rt_rr_tasks.first().copied();
        }
        
        // CFS tasks (lowest vruntime first)
        if let Some((&vruntime, &task_id)) = self.cfs_tasks.iter().next() {
            self.min_vruntime = vruntime;
            return Some(task_id);
        }
        
        // Idle tasks
        if !self.idle_tasks.is_empty() {
            return self.idle_tasks.first().copied();
        }
        
        None
    }
    
    /// Update load information
    pub(crate) fn update_load(&mut self) {
        self.load.runnable = self.nr_running;
        self.load.weight = self.calculate_weight();
        // Simple moving average
        self.load.avg_load = (self.load.avg_load * 0.9) + (self.load.runnable as f32 * 0.1);
    }
    
    /// Calculate total weight of tasks
    fn calculate_weight(&self) -> u64 {
        // Sum of task weights (nice value -> weight)
        // Simplified: each task has weight based on priority
        self.nr_running as u64 * 1024
    }
}
