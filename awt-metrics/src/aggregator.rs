use alloc::format;
use alloc::rc::Rc;
use core::fmt::{Debug, Display, Formatter, Result};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;

use awt_simulation::request::{Request, Status};

use crate::{Metric, MetricType};

#[derive(Default)]
pub struct Aggregator {
    metrics: HashMap<MetricType, Metric>,
}

fn report(m: &mut Metric, r: &Ref<'_, Request>) {
    match m.metric() {
        MetricType::ServiceLevel(ticks) if &Status::Answered == r.status() => {
            if let Some(tick) = r.wait_time() {
                m.report_bool(tick <= ticks);
            }
        }
        MetricType::AverageWorkTime if &Status::Answered == r.status() => {
            if let Some(tick) = r.handle_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AverageSpeedAnswer if &Status::Answered == r.status() => {
            if let Some(tick) = r.wait_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AverageTimeToAbandon if &Status::Abandoned == r.status() => {
            if let Some(tick) = r.wait_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AbandonRate => m.report_bool(&Status::Abandoned == r.status()),
        MetricType::AverageTimeInQueue => {
            if let Some(tick) = r.wait_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AnswerCount if &Status::Answered == r.status() => m.report(),
        MetricType::UtilisationTime => todo!(),
        _ => (),
    }
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

    pub fn push(&mut self, m: Metric) {
        self.metrics.insert(m.metric(), m);
    }

    pub fn calculate(&mut self, requests: &[Rc<RefCell<Request>>]) {
        for request in requests {
            for metric in &mut self.metrics.values_mut() {
                report(metric, &request.borrow());
            }
        }
    }
}
