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
    /// Count of Answered
    AnswerCount,
}

#[derive(Clone, Debug)]
pub enum Aggregate {
    Meanable(Meanable),
    Countable(Countable),
    Percentable(Percentable),
}

// Meanable are metrics which we count a total of ticks for, and we want the average of those values
// Report: Just provide a value. a usize report_usize(value: usize)
#[derive(Clone, Debug)]
pub struct Meanable {
    sum: usize,
    count: usize,
    target: f64,
}

impl Meanable {
    pub fn report_usize(&mut self, value: usize) {
        self.sum += value;
        self.count += 1;
    }

    pub fn with_target(target: f64) -> Self {
        Self {
            sum: 0,
            count: 0,
            target,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn value(&self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }

        let sum = self.sum as f64;
        let count = self.sum as f64;

        Some(sum / count)
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn on_target(&self) -> bool {
        match self.count {
            0 => false,
            _ => self.target < (self.sum as f64 / self.count as f64),
        }
    }
}

// Countable are metrics which we only want to count. e.g we have X requests
// Report: Just report()
#[derive(Clone, Debug)]
pub struct Countable {
    count: usize,
    target: usize,
}

impl Countable {
    pub fn report(&mut self) {
        self.count += 1;
    }

    pub fn with_target(target: usize) -> Self {
        Self { count: 0, target }
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn value(&self) -> Option<usize> {
        Some(self.count)
    }

    pub fn on_target(&self) -> bool {
        match self.count {
            0 => false,
            _ => self.target < self.count,
        }
    }
}

// Percentable is a count of matching values against a total available
// Report: Position and negative. e.g. Things that in bounds, and things that
//         out of bounds
//         report_bool(value: bool)
#[derive(Clone, Debug)]
pub struct Percentable {
    sum: usize,
    count: usize,
    target: f64,
}

impl Percentable {
    pub fn report_bool(&mut self, value: bool) {
        if value {
            self.sum += 1;
        }
        self.count += 1;
    }

    pub fn with_target(target: f64) -> Self {
        Self {
            sum: 0,
            count: 0,
            target,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn value(&self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }
        let sum = self.sum as f64;
        let count = self.count as f64;

        Some(sum / count)
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn on_target(&self) -> bool {
        match self.count {
            0 => false,
            _ => self.target < (self.sum as f64 / self.count as f64),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Metric {
    metric_type: MetricType,
    aggregate: Aggregate,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct MetricError {}

// Structure and setup
impl Metric {
    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn with_target_f64(metric_type: MetricType, target: f64) -> Result<Self, MetricError> {
        match metric_type {
            MetricType::AverageWorkTime
            | MetricType::AverageSpeedAnswer
            | MetricType::AverageTimeInQueue
            | MetricType::AverageTimeToAbandon => Ok(Self {
                metric_type,
                aggregate: Aggregate::Meanable(Meanable::with_target(target)),
            }),
            MetricType::UtilisationTime | MetricType::ServiceLevel(_) | MetricType::AbandonRate => {
                Ok(Self {
                    metric_type,
                    aggregate: Aggregate::Percentable(Percentable::with_target(target)),
                })
            }
            _ => Err(MetricError {}),
        }
    }

    pub fn with_target_usize(metric_type: MetricType, target: usize) -> Result<Self, MetricError> {
        match metric_type {
            MetricType::AnswerCount => Ok(Self {
                metric_type,
                aggregate: Aggregate::Countable(Countable::with_target(target)),
            }),
            _ => Err(MetricError {}),
        }
    }

    pub fn metric(&self) -> MetricType {
        self.metric_type
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn value(&self) -> Option<f64> {
        match &self.aggregate {
            Aggregate::Meanable(a) => a.value(),
            Aggregate::Percentable(a) => a.value(),
            Aggregate::Countable(a) => a.value().map(|v| v as f64),
        }
    }

    pub fn on_target(&self) -> bool {
        match &self.aggregate {
            Aggregate::Meanable(a) => a.on_target(),
            Aggregate::Percentable(a) => a.on_target(),
            Aggregate::Countable(a) => a.on_target(),
        }
    }
}

// Reporting functions
impl Metric {
    pub fn report(&mut self) {
        match &mut self.aggregate {
            Aggregate::Countable(a) => a.report(),
            _ => todo!(),
        }
    }

    pub fn report_bool(&mut self, value: bool) {
        match &mut self.aggregate {
            Aggregate::Percentable(a) => a.report_bool(value),
            _ => todo!(),
        }
    }

    pub fn report_usize(&mut self, value: usize) {
        match &mut self.aggregate {
            Aggregate::Meanable(a) => a.report_usize(value),
            _ => todo!(),
        }
    }
}
