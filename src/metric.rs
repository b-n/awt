// TODO: Metrics come in different flavours. The metrics should really wrap these types since they
// have different targets etc e.g.
// - Percentile (ServiceLevel + Abandonrate)
// - Mean (Average*)
// - Countable (# of answered, # of abandoned, etc)

/// Enumerates a metric to trace on a `Request`.
#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MetricType {
    /// Percent of `Client`s answered in `tick`.
    ServiceLevel(usize),
    /// Mean of work time of answered `Request`.
    AverageWorkTime,
    /// Mean for `tick` of answered `Request`.
    AverageSpeedAnswer,
    /// Mean of `tick` of abandoned `Request` (at abandon).
    AverageTimeToAbandon,
    /// Percent of `Request` abandoned vs. total `Request` count.
    AbandonRate,
    /// Mean of `tick` of `Request` in queue for both answered and abandoned.
    AverageTimeInQueue,
    /// Percent of `Server` time spent answering `Request`s.
    UtilisationTime,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Metric {
    metric: MetricType,
    sum: f64,
    count: f64,
    target: f64,
}

impl Metric {
    fn new(t: MetricType) -> Self {
        Self {
            metric: t,
            sum: 0.0,
            count: 0.0,
            target: 0f64,
        }
    }

    pub fn with_target(t: MetricType, target: f64) -> Self {
        Self {
            target,
            ..Self::new(t)
        }
    }

    pub fn metric(&self) -> &MetricType {
        &self.metric
    }
}

impl Metric {
    pub fn report_bool(&mut self, value: bool) {
        self.sum += if value { 1.0 } else { 0.0 };
        self.count += 1.0;
    }

    // TODO: We could use std::time::Duration instead of `tick`
    #[allow(clippy::cast_precision_loss)]
    pub fn report_tick(&mut self, value: usize) {
        self.sum += value as f64;
        self.count += 1.0;
    }

    pub fn value(&self) -> f64 {
        if self.count == 0.0 {
            return 0.0;
        }

        self.sum / self.count
    }
}
