use std::sync::Mutex;
use std::time::Instant;

use sqlx::PgPool;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use domain::info::{
    CpuStats, DbPoolStats, MemoryStats, ProcessStats, RuntimeStats, RuntimeStatsProvider,
    TokioStats,
};

pub struct DefaultRuntimeStatsProvider {
    system: Mutex<System>,
    pid: Pid,
    pool: PgPool,
    start_time: Instant,
    cores: u32,
}

impl DefaultRuntimeStatsProvider {
    pub fn new(pool: PgPool) -> Self {
        let pid = sysinfo::get_current_pid().expect("current pid must be available");
        let mut system = System::new();
        let refresh = ProcessRefreshKind::nothing().with_cpu().with_memory();
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, refresh);
        let cores = u32::try_from(System::physical_core_count().unwrap_or(1))
            .unwrap_or(1)
            .max(1);
        Self {
            system: Mutex::new(system),
            pid,
            pool,
            start_time: Instant::now(),
            cores,
        }
    }
}

#[async_trait::async_trait]
impl RuntimeStatsProvider for DefaultRuntimeStatsProvider {
    #[tracing::instrument(level = "trace", skip(self))]
    async fn snapshot(&self) -> RuntimeStats {
        let (memory, cpu_proc, threads) = {
            let mut sys = self.system.lock().expect("sysinfo mutex must not be poisoned");
            let refresh = ProcessRefreshKind::nothing().with_cpu().with_memory();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[self.pid]),
                true,
                refresh,
            );
            let proc = sys.process(self.pid);
            let (resident, virt, cpu, threads) = proc
                .map(|p| {
                    (
                        p.memory(),
                        p.virtual_memory(),
                        p.cpu_usage(),
                        p.tasks().map_or(0, |t| t.len()),
                    )
                })
                .unwrap_or((0, 0, 0.0, 0));
            (
                MemoryStats {
                    resident_bytes: resident,
                    virtual_bytes: virt,
                },
                cpu,
                u32::try_from(threads).unwrap_or(0),
            )
        };

        let cpu_total = cpu_proc / self.cores as f32;

        let tokio_metrics = tokio::runtime::Handle::current().metrics();
        let tokio_stats = TokioStats {
            worker_threads: u32::try_from(tokio_metrics.num_workers()).unwrap_or(0),
            // num_blocking_threads requires tokio_unstable; unavailable in this build
            blocking_threads: 0,
        };

        let pool_size = self.pool.size();
        let pool_idle = u32::try_from(self.pool.num_idle()).unwrap_or(0);
        let db_pool = DbPoolStats {
            size: pool_size,
            idle: pool_idle,
            active: pool_size.saturating_sub(pool_idle),
            max: self.pool.options().get_max_connections(),
        };

        RuntimeStats {
            memory,
            cpu: CpuStats {
                cores: self.cores,
                usage_percent: cpu_total,
            },
            tokio: tokio_stats,
            db_pool,
            process: ProcessStats {
                uptime_seconds: self.start_time.elapsed().as_secs(),
                threads,
            },
            timestamp: chrono::Utc::now(),
        }
    }
}
