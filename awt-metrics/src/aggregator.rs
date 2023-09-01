use alloc::format;
use core::fmt::{Debug, Display, Formatter, Result};
use std::collections::HashMap;

use awt_simulation::request::Data as RequestData;

use crate::{Metric, MetricType};

#[derive(Default)]
pub struct Aggregator {
    metrics: HashMap<MetricType, Metric>,
}

impl Display for Aggregator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for metric in self.metrics.values() {
            writeln!(
                f,
                "{:20} {:<5} {}",
                format!("{:?}", metric.metric()),
                metric.on_target(),
                metric
            )?;
        }
        Ok(())
    }
}

impl Debug for Aggregator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for metric in &self.metrics {
            writeln!(f, "{metric:?}")?;
        }
        Ok(())
    }
}

impl Aggregator {
    #[must_use]
    pub fn with_metrics(metrics: &[Metric]) -> Self {
        let metrics = metrics.iter().map(|m| (m.metric(), m.clone())).collect();
        Self { metrics }
    }

    pub fn clean(&mut self) {}

    pub fn push(&mut self, m: Metric) {
        self.metrics.insert(m.metric(), m);
    }

    pub fn calculate(&mut self, request_data: &[RequestData]) {
        for request in request_data {
            for metric in &mut self.metrics.values_mut() {
                metric.report(request);
            }
        }
    }
}
