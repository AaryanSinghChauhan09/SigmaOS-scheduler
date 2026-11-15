//! Load balancing implementation
//!
//! CPU load balancing across runqueues

use crate::runqueue::RunQueue;

/// Load balancer
pub struct LoadBalancer {
    pub num_cpus: usize,
    pub balance_interval: u64,
    pub imbalance_threshold: f32,
}

impl LoadBalancer {
    /// Create new load balancer
    pub fn new(num_cpus: usize) -> Self {
        Self {
            num_cpus,
            balance_interval: 100_000_000, // 100ms in nanoseconds
            imbalance_threshold: 0.25, // 25% imbalance threshold
        }
    }
    
    /// Balance load across runqueues
    pub fn balance(&mut self, runqueues: &mut [RunQueue]) {
        // Find most loaded and least loaded CPUs
        let mut max_load = 0.0f32;
        let mut min_load = f32::MAX;
        let mut max_cpu = 0;
        let mut min_cpu = 0;
        
        for (i, rq) in runqueues.iter().enumerate() {
            let load = rq.load.avg_load;
            if load > max_load {
                max_load = load;
                max_cpu = i;
            }
            if load < min_load {
                min_load = load;
                min_cpu = i;
            }
        }
        
        // Check if imbalance exceeds threshold
        let imbalance = (max_load - min_load) / max_load.max(1.0);
        
        if imbalance > self.imbalance_threshold {
            // Move tasks from max_cpu to min_cpu
            self.move_tasks(runqueues, max_cpu, min_cpu);
        }
    }
    
    /// Move tasks from source to destination CPU
    fn move_tasks(&self, runqueues: &mut [RunQueue], source: usize, dest: usize) {
        if source >= runqueues.len() || dest >= runqueues.len() {
            return;
        }
        
        // Move one task at a time (simplified)
        if let Some((&vruntime, &task_id)) = runqueues[source].cfs_tasks.iter().next() {
            // Remove from source
            runqueues[source].cfs_tasks.remove(&vruntime);
            runqueues[source].nr_running = runqueues[source].nr_running.saturating_sub(1);
            
            // Add to destination
            runqueues[dest].cfs_tasks.insert(vruntime, task_id);
            runqueues[dest].nr_running += 1;
            
            // Update load info
            runqueues[source].update_load();
            runqueues[dest].update_load();
        }
    }
    
    /// Get average load across all CPUs
    pub fn average_load(&self, runqueues: &[RunQueue]) -> f32 {
        if runqueues.is_empty() {
            return 0.0;
        }
        
        let total_load: f32 = runqueues.iter().map(|rq| rq.load.avg_load).sum();
        total_load / runqueues.len() as f32
    }
    
    /// Check if system is balanced
    pub fn is_balanced(&self, runqueues: &[RunQueue]) -> bool {
        let avg_load = self.average_load(runqueues);
        
        for rq in runqueues {
            let deviation = (rq.load.avg_load - avg_load).abs() / avg_load.max(1.0);
            if deviation > self.imbalance_threshold {
                return false;
            }
        }
        
        true
    }
}
