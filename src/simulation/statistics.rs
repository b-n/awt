pub use core::fmt::{Debug, Display};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use super::{Request, RequestStatus};
use crate::{Metric, MetricType};

#[derive(Default)]
pub struct Statistics {
    metrics: HashMap<MetricType, Metric>,
    calculated: bool,
}

fn report(m: &mut Metric, r: &Ref<'_, Request>) {
    match m.metric() {
        MetricType::ServiceLevel(ticks) if &RequestStatus::Answered == r.status() => {
            if let Some(tick) = r.wait_time() {
                m.report_bool(tick <= ticks);
            }
        }
        MetricType::AverageWorkTime if &RequestStatus::Answered == r.status() => {
            if let Some(tick) = r.handle_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AverageSpeedAnswer if &RequestStatus::Answered == r.status() => {
            if let Some(tick) = r.wait_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AverageTimeToAbandon if &RequestStatus::Abandoned == r.status() => {
            if let Some(tick) = r.wait_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AbandonRate => m.report_bool(&RequestStatus::Abandoned == r.status()),
        MetricType::AverageTimeInQueue => {
            if let Some(tick) = r.wait_time() {
                m.report_duration(tick);
            }
        }
        MetricType::AnswerCount if &RequestStatus::Answered == r.status() => m.report(),
        MetricType::UtilisationTime => todo!(),
        _ => (),
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Debug for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for metric in &self.metrics {
            writeln!(f, "{metric:?}")?;
        }
        Ok(())
    }
}

impl Statistics {
    pub fn push(&mut self, m: Metric) {
        self.metrics.insert(m.metric(), m);
    }

    #[allow(dead_code)]
    pub fn get(&self, m: &MetricType) -> Option<&Metric> {
        self.metrics.get(m)
    }

    pub fn calculate(&mut self, requests: &Vec<Rc<RefCell<Request>>>) {
        if self.calculated {
            return;
        }

        for request in requests {
            for metric in &mut self.metrics.values_mut() {
                report(metric, &request.borrow());
            }
        }
    }
}
