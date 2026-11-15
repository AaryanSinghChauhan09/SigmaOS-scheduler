//! Completely Fair Scheduler (CFS) implementation
//!
//! Fair CPU time distribution based on vruntime

use crate::task::{Task, TaskId};
use crate::runqueue::RunQueue;

/// CFS scheduler
pub struct CfsScheduler {
    pub runqueue: RunQueue,
    pub min_vruntime: u64,
    pub curr_task: Option<TaskId>,
}

impl CfsScheduler {
    /// Create new CFS scheduler
    pub fn new(cpu_id: usize) -> Self {
        Self {
            runqueue: RunQueue::new(cpu_id),
            min_vruntime: 0,
            curr_task: None,
        }
    }
    
    /// Enqueue task
    pub fn enqueue_task(&mut self, task: &Task) {
        // Update min_vruntime
        if task.vruntime > self.min_vruntime {
            self.min_vruntime = task.vruntime;
        }
        
        // Add to runqueue
        self.runqueue.enqueue(task.clone());
    }
    
    /// Dequeue task
    pub fn dequeue_task(&mut self, task_id: TaskId) -> bool {
        self.runqueue.dequeue(task_id)
    }
    
    /// Pick next task (lowest vruntime)
    pub fn pick_next_task(&mut self) -> Option<TaskId> {
        // Get task with lowest vruntime
        if let Some((&vruntime, &task_id)) = self.runqueue.cfs_tasks.iter().next() {
            self.min_vruntime = vruntime;
            self.curr_task = Some(task_id);
            return Some(task_id);
        }
        None
    }
    
    /// Task tick (update vruntime)
    pub fn task_tick(&mut self, task: &mut Task, runtime_ns: u64) {
        // Update vruntime based on nice value
        let nice_weight = calculate_nice_weight(task.priority);
        let weight = 1024; // Base weight
        
        // vruntime += runtime * weight / nice_weight
        let delta = (runtime_ns * weight) / nice_weight;
        task.vruntime += delta;
        
        // Requeue task if still runnable
        if task.state == crate::task::TaskState::Runnable {
            self.enqueue_task(task);
        }
    }
    
    /// Check if should preempt current task
    pub fn check_preempt_curr(&self, task: &Task) -> bool {
        // Preempt if new task has lower vruntime
        if let Some(curr_id) = self.curr_task {
            if let Some((_, &next_id)) = self.runqueue.cfs_tasks.iter().next() {
                if next_id != curr_id {
                    return true;
                }
            }
        }
        false
    }
}

/// Calculate nice weight from nice value
fn calculate_nice_weight(nice: i32) -> u64 {
    // Linux kernel weight table (simplified)
    const NICE_WEIGHT: [u64; 40] = [
        88761, 71755, 56483, 46273, 36291,
        29154, 23254, 18705, 14949, 11916,
        9548, 7620, 6100, 4904, 3906,
        3121, 2501, 1991, 1586, 1277,
        1024, 820, 655, 526, 423,
        335, 272, 215, 172, 137,
        110, 87, 70, 56, 45,
        36, 29, 23, 18, 15,
    ];
    
    let idx = (nice + 20) as usize;
    if idx < NICE_WEIGHT.len() {
        NICE_WEIGHT[idx]
    } else {
        1024
    }
}
