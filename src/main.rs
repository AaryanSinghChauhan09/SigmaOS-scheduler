//! Demo for SigmaOS Scheduler
//!
//! This binary demonstrates scheduler functionality

use sigmaos_scheduler::*;
use anyhow::Result;

fn main() -> Result<()> {
    println!("SigmaOS Scheduler Demo");
    println!("====================");
    
    // Create scheduler with 4 CPUs
    let num_cpus = 4;
    let mut scheduler = Scheduler::new(num_cpus)?;
    scheduler.init()?;
    
    println!("Scheduler initialized with {} CPUs", num_cpus);
    
    // Create tasks with different policies
    let task1 = Task::new(1000, SchedulerPolicy::Normal, 0); // Nice 0
    let task2 = Task::new(1001, SchedulerPolicy::Normal, -5); // Nice -5 (higher priority)
    let task3 = Task::new(1002, SchedulerPolicy::RealTimeFifo, 0);
    task3.rt_priority = Some(50);
    let task4 = Task::new(1003, SchedulerPolicy::Deadline, 0);
    
    // Set deadline parameters for task4
    let mut task4 = task4;
    task4.runtime = 1_000_000; // 1ms
    task4.period = 10_000_000; // 10ms
    task4.deadline = 5_000_000; // 5ms
    
    // Enqueue tasks
    let task1_id = scheduler.enqueue_task(task1)?;
    let task2_id = scheduler.enqueue_task(task2)?;
    let task3_id = scheduler.enqueue_task(task3)?;
    let task4_id = scheduler.enqueue_task(task4)?;
    
    println!("Created 4 tasks:");
    println!("  Task 1: Normal priority 0 (ID: {})", task1_id);
    println!("  Task 2: Normal priority -5 (ID: {})", task2_id);
    println!("  Task 3: Real-time FIFO priority 50 (ID: {})", task3_id);
    println!("  Task 4: Deadline (ID: {})", task4_id);
    
    // Simulate scheduling
    println!("\nSimulating scheduling...");
    
    for _ in 0..10 {
        if let Some(task_id) = scheduler.pick_next_task()? {
            println!("  Running task {}", task_id);
            scheduler.run_task(task_id)?;
        }
    }
    
    // Set CPU affinity for task1
    let mut affinity = CpuSet::new(num_cpus);
    affinity.set_cpu(0);
    affinity.set_cpu(1);
    scheduler.set_affinity(task1_id, affinity)?;
    
    println!("\nSet CPU affinity for task {} to CPUs 0,1", task1_id);
    
    // Get load information
    println!("\nLoad information:");
    for cpu_id in 0..num_cpus {
        let load = scheduler.get_load(cpu_id)?;
        println!("  CPU {}: {} runnable tasks, avg_load: {:.2}", 
                 cpu_id, load.runnable, load.avg_load);
    }
    
    // Balance load
    scheduler.balance_load()?;
    println!("\nLoad balanced");
    
    // Print scheduler statistics
    println!("\nScheduler Statistics:");
    println!("  Total tasks: {}", scheduler.tasks.len());
    println!("  Total CPUs: {}", scheduler.num_cpus);
    println!("  Next task ID: {}", scheduler.next_task_id);
    
    Ok(())
}
