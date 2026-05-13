use serde::Serialize;
use utoipa::ToSchema;

use domain::info::RuntimeStats;

#[derive(Debug, Serialize, ToSchema)]
pub struct RuntimeStatsResponse {
    pub memory: MemoryStatsResponse,
    pub cpu: CpuStatsResponse,
    pub tokio: TokioStatsResponse,
    #[serde(rename = "dbPool")]
    pub db_pool: DbPoolStatsResponse,
    pub process: ProcessStatsResponse,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MemoryStatsResponse {
    #[serde(rename = "residentBytes")]
    pub resident_bytes: u64,
    #[serde(rename = "virtualBytes")]
    pub virtual_bytes: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CpuStatsResponse {
    pub cores: u32,
    #[serde(rename = "usagePercent")]
    pub usage_percent: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokioStatsResponse {
    #[serde(rename = "workerThreads")]
    pub worker_threads: u32,
    #[serde(rename = "blockingThreads")]
    pub blocking_threads: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DbPoolStatsResponse {
    pub size: u32,
    pub idle: u32,
    pub active: u32,
    pub max: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProcessStatsResponse {
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: u64,
    pub threads: u32,
}

impl From<&RuntimeStats> for RuntimeStatsResponse {
    fn from(value: &RuntimeStats) -> Self {
        Self {
            memory: MemoryStatsResponse {
                resident_bytes: value.memory.resident_bytes,
                virtual_bytes: value.memory.virtual_bytes,
            },
            cpu: CpuStatsResponse {
                cores: value.cpu.cores,
                usage_percent: value.cpu.usage_percent,
            },
            tokio: TokioStatsResponse {
                worker_threads: value.tokio.worker_threads,
                blocking_threads: value.tokio.blocking_threads,
            },
            db_pool: DbPoolStatsResponse {
                size: value.db_pool.size,
                idle: value.db_pool.idle,
                active: value.db_pool.active,
                max: value.db_pool.max,
            },
            process: ProcessStatsResponse {
                uptime_seconds: value.process.uptime_seconds,
                threads: value.process.threads,
            },
            timestamp: value.timestamp.timestamp_millis(),
        }
    }
}
