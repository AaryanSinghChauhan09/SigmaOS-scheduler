//! Earliest Eligible Virtual Deadline First (EEVDF) scheduler
//!
//! Improved fair scheduling algorithm

use crate::task::{Task, TaskId};
use crate::runqueue::RunQueue;

/// EEVDF scheduler
pub struct EevdfScheduler {
    pub runqueue: RunQueue,
    pub current_task: Option<TaskId>,
    pub lag: u64,
}

impl EevdfScheduler {
    /// Create new EEVDF scheduler
    pub fn new(cpu_id: usize) -> Self {
        Self {
            runqueue: RunQueue::new(cpu_id),
            current_task: None,
            lag: 0,
        }
    }
    
    /// Enqueue task
    pub fn enqueue_task(&mut self, task: &Task) {
        self.runqueue.enqueue(task.clone());
    }
    
    /// Dequeue task
    pub fn dequeue_task(&mut self, task_id: TaskId) -> bool {
        self.runqueue.dequeue(task_id)
    }
    
    /// Pick next task (earliest eligible virtual deadline)
    pub fn pick_next_task(&mut self) -> Option<TaskId> {
        // Find task with earliest virtual deadline
        // Virtual deadline = vruntime + time_slice / weight
        
        let mut best_task: Option<(u64, TaskId)> = None;
        
        for (&vruntime, &task_id) in &self.runqueue.cfs_tasks {
            // Calculate virtual deadline
            let time_slice = 6_000_000; // 6ms base
            let weight = 1024; // Base weight
            let virtual_deadline = vruntime + (time_slice * weight) / weight;
            
            if best_task.is_none() || virtual_deadline < best_task.unwrap().0 {
                best_task = Some((virtual_deadline, task_id));
            }
        }
        
        if let Some((_, task_id)) = best_task {
            self.current_task = Some(task_id);
            return Some(task_id);
        }
        
        None
    }
    
    /// Task tick (update virtual time)
    pub fn task_tick(&mut self, task: &mut Task, runtime_ns: u64) {
        // Update vruntime with lag compensation
        let nice_weight = calculate_nice_weight(task.priority);
        let weight = 1024;
        
        // Apply lag to vruntime calculation
        let effective_runtime = runtime_ns + self.lag;
        let delta = (effective_runtime * weight) / nice_weight;
        task.vruntime += delta;
        
        // Decay lag
        self.lag = self.lag.saturating_sub(runtime_ns / 10);
        
        // Requeue if still runnable
        if task.state == crate::task::TaskState::Runnable {
            self.enqueue_task(task);
        }
    }
}

/// Calculate nice weight from nice value
fn calculate_nice_weight(nice: i32) -> u64 {
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
