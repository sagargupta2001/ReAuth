use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Mutex;

const LATENCY_BUCKETS_MS: &[u64] = &[5, 10, 25, 50, 100, 250, 500, 1_000, 2_500, 5_000, 10_000];

#[derive(Default)]
struct MetricsInner {
    request_count: u64,
    status_2xx: u64,
    status_3xx: u64,
    status_4xx: u64,
    status_5xx: u64,
    latency_sum_ms: u64,
    latency_buckets: Vec<u64>,
    latency_overflow: u64,
}

pub struct MetricsService {
    started_at: DateTime<Utc>,
    inner: Mutex<MetricsInner>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub since: String,
    pub request_count: u64,
    pub status_counts: StatusCounts,
    pub latency_ms: LatencyHistogram,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusCounts {
    pub success: u64,
    pub redirect: u64,
    pub client_error: u64,
    pub server_error: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyHistogram {
    pub buckets: Vec<LatencyBucket>,
    pub overflow_count: u64,
    pub count: u64,
    pub sum_ms: u64,
    pub avg_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyBucket {
    pub le: u64,
    pub count: u64,
}

impl MetricsService {
    pub fn new() -> Self {
        Self {
            started_at: Utc::now(),
            inner: Mutex::new(MetricsInner {
                latency_buckets: vec![0; LATENCY_BUCKETS_MS.len()],
                ..MetricsInner::default()
            }),
        }
    }

    pub fn record_request(&self, duration_ms: u64, status: u16) {
        let mut inner = self.inner.lock().unwrap();
        inner.request_count += 1;
        inner.latency_sum_ms = inner.latency_sum_ms.saturating_add(duration_ms);

        match status / 100 {
            2 => inner.status_2xx += 1,
            3 => inner.status_3xx += 1,
            4 => inner.status_4xx += 1,
            5 => inner.status_5xx += 1,
            _ => {}
        }

        let mut bucketed = false;
        for (idx, bucket) in LATENCY_BUCKETS_MS.iter().enumerate() {
            if duration_ms <= *bucket {
                inner.latency_buckets[idx] += 1;
                bucketed = true;
                break;
            }
        }
        if !bucketed {
            inner.latency_overflow += 1;
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let inner = self.inner.lock().unwrap();
        let count = inner.request_count;
        let avg_ms = if count == 0 {
            0.0
        } else {
            inner.latency_sum_ms as f64 / count as f64
        };

        let buckets = LATENCY_BUCKETS_MS
            .iter()
            .enumerate()
            .map(|(idx, bucket)| LatencyBucket {
                le: *bucket,
                count: inner.latency_buckets[idx],
            })
            .collect();

        MetricsSnapshot {
            since: self.started_at.to_rfc3339(),
            request_count: inner.request_count,
            status_counts: StatusCounts {
                success: inner.status_2xx,
                redirect: inner.status_3xx,
                client_error: inner.status_4xx,
                server_error: inner.status_5xx,
            },
            latency_ms: LatencyHistogram {
                buckets,
                overflow_count: inner.latency_overflow,
                count,
                sum_ms: inner.latency_sum_ms,
                avg_ms,
            },
        }
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
}
