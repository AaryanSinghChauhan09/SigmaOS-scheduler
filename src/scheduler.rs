//! Scheduler implementation
//!
//! Main scheduler class with multiple scheduling policies

use anyhow::Result;
use std::collections::HashMap;

/// Task ID type
pub type TaskId = u64;

/// Scheduler
pub struct Scheduler {
    pub num_cpus: usize,
    pub runqueues: Vec<RunQueue>,
    pub tasks: HashMap<TaskId, Task>,
    pub next_task_id: TaskId,
}

impl Scheduler {
    /// Create new scheduler
    pub fn new(num_cpus: usize) -> Result<Self> {
        let mut runqueues = Vec::new();
        for i in 0..num_cpus {
            runqueues.push(RunQueue::new(i));
        }
        
        Ok(Self {
            num_cpus,
            runqueues,
            tasks: HashMap::new(),
            next_task_id: 1,
        })
    }
    
    /// Initialize scheduler
    pub fn init(&mut self) -> Result<()> {
        // Initialize scheduler classes
        Ok(())
    }
    
    /// Enqueue task
    pub fn enqueue_task(&mut self, task: Task) -> Result<TaskId> {
        let task_id = self.next_task_id;
        self.next_task_id += 1;
        
        let mut task = task;
        task.id = task_id;
        
        self.tasks.insert(task_id, task.clone());
        
        // Select runqueue based on policy and affinity
        let cpu_id = self.select_runqueue(&task);
        self.runqueues[cpu_id].enqueue(task);
        
        Ok(task_id)
    }
    
    /// Dequeue task
    pub fn dequeue_task(&mut self, task_id: TaskId) -> Result<Task> {
        if let Some(task) = self.tasks.remove(&task_id) {
            // Remove from runqueue
            for rq in &mut self.runqueues {
                rq.dequeue(task_id);
            }
            Ok(task)
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }
    
    /// Pick next task to run
    pub fn pick_next_task(&mut self) -> Result<Option<TaskId>> {
        // For each runqueue, pick next task based on policy
        for rq in &mut self.runqueues {
            if let Some(task_id) = rq.pick_next() {
                if let Some(task) = self.tasks.get(&task_id) {
                    return Ok(Some(task_id));
                }
            }
        }
        Ok(None)
    }
    
    /// Run task
    pub fn run_task(&mut self, task_id: TaskId) -> Result<()> {
        // Execute task for its time slice
        if let Some(task) = self.tasks.get_mut(&task_id) {
            // Update vruntime, runtime, etc.
            task.vruntime += task.get_time_slice();
        }
        Ok(())
    }
    
    /// Set task priority
    pub fn set_priority(&mut self, task_id: TaskId, priority: i32) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.priority = priority;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }
    
    /// Set real-time priority
    pub fn set_rt_priority(&mut self, task_id: TaskId, priority: i32) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            if priority < 1 || priority > 99 {
                return Err(anyhow::anyhow!("Invalid RT priority"));
            }
            task.rt_priority = Some(priority);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }
    
    /// Set deadline parameters
    pub fn set_deadline(&mut self, task_id: TaskId, runtime: u64, period: u64, deadline: u64) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.runtime = runtime;
            task.period = period;
            task.deadline = deadline;
            task.policy = SchedulerPolicy::Deadline;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }
    
    /// Set CPU affinity
    pub fn set_affinity(&mut self, task_id: TaskId, affinity: CpuSet) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.affinity = affinity;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }
    
    /// Get CPU affinity
    pub fn get_affinity(&self, task_id: TaskId) -> Result<CpuSet> {
        if let Some(task) = self.tasks.get(&task_id) {
            Ok(task.affinity.clone())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }
    
    /// Balance load across CPUs
    pub fn balance_load(&mut self) -> Result<()> {
        // Simple load balancing: move tasks from overloaded to underloaded CPUs
        // In real implementation, would use more sophisticated algorithm
        Ok(())
    }
    
    /// Get load information for CPU
    pub fn get_load(&self, cpu_id: usize) -> Result<LoadInfo> {
        if cpu_id >= self.runqueues.len() {
            return Err(anyhow::anyhow!("Invalid CPU ID"));
        }
        Ok(self.runqueues[cpu_id].load.clone())
    }
    
    /// Select runqueue for task
    fn select_runqueue(&self, task: &Task) -> usize {
        // Use affinity if set, otherwise round-robin
        if !task.affinity.is_empty() {
            // Find first allowed CPU
            for cpu in 0..self.num_cpus {
                if task.affinity.has_cpu(cpu) {
                    return cpu;
                }
            }
        }
        // Round-robin fallback
        (task.id as usize) % self.num_cpus
    }
}
