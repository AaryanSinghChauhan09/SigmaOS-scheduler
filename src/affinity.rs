//! CPU affinity management
//!
//! CPU affinity mask management

use serde::{Serialize, Deserialize};

/// CPU set (affinity mask)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSet {
    pub bits: Vec<u64>,
    pub num_cpus: usize,
}

impl CpuSet {
    /// Create new CPU set with all CPUs allowed
    pub fn all() -> Self {
        Self {
            bits: vec![u64::MAX],
            num_cpus: 64, // Default max CPUs
        }
    }
    
    /// Create new CPU set with specific CPUs
    pub fn new(num_cpus: usize) -> Self {
        let num_words = (num_cpus + 63) / 64;
        Self {
            bits: vec![0; num_words],
            num_cpus,
        }
    }
    
    /// Set CPU in mask
    pub fn set_cpu(&mut self, cpu: usize) {
        if cpu < self.num_cpus {
            let word = cpu / 64;
            let bit = cpu % 64;
            if word < self.bits.len() {
                self.bits[word] |= 1 << bit;
            }
        }
    }
    
    /// Clear CPU from mask
    pub fn clear_cpu(&mut self, cpu: usize) {
        if cpu < self.num_cpus {
            let word = cpu / 64;
            let bit = cpu % 64;
            if word < self.bits.len() {
                self.bits[word] &= !(1 << bit);
            }
        }
    }
    
    /// Check if CPU is in mask
    pub fn has_cpu(&self, cpu: usize) -> bool {
        if cpu >= self.num_cpus {
            return false;
        }
        let word = cpu / 64;
        let bit = cpu % 64;
        if word < self.bits.len() {
            (self.bits[word] & (1 << bit)) != 0
        } else {
            false
        }
    }
    
    /// Check if mask is empty
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&word| word == 0)
    }
    
    /// Get count of CPUs in mask
    pub fn count(&self) -> usize {
        self.bits.iter().map(|&word| word.count_ones() as usize).sum()
    }
    
    /// Get first CPU in mask
    pub fn first(&self) -> Option<usize> {
        for (word_idx, &word) in self.bits.iter().enumerate() {
            if word != 0 {
                let bit_idx = word.trailing_zeros() as usize;
                let cpu = word_idx * 64 + bit_idx;
                if cpu < self.num_cpus {
                    return Some(cpu);
                }
            }
        }
        None
    }
    
    /// Iterate over CPUs in mask
    pub fn iter(&self) -> CpuSetIter<'_> {
        CpuSetIter {
            cpu_set: self,
            current: 0,
        }
    }
}

/// CPU set iterator
pub struct CpuSetIter<'a> {
    cpu_set: &'a CpuSet,
    current: usize,
}

impl<'a> Iterator for CpuSetIter<'a> {
    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.cpu_set.num_cpus {
            if self.cpu_set.has_cpu(self.current) {
                let cpu = self.current;
                self.current += 1;
                return Some(cpu);
            }
            self.current += 1;
        }
        None
    }
}
