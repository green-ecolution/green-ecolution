//! Live runtime stats pushed over `/v1/ws/stats`.

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub memory: MemoryStats,
    pub cpu: CpuStats,
    pub tokio: TokioStats,
    pub db_pool: DbPoolStats,
    pub process: ProcessStats,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub resident_bytes: u64,
    pub virtual_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct CpuStats {
    pub cores: u32,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct TokioStats {
    pub worker_threads: u32,
    pub blocking_threads: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct DbPoolStats {
    pub size: u32,
    pub idle: u32,
    pub active: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessStats {
    pub uptime_seconds: u64,
    pub threads: u32,
}
