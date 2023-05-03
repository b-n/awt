use core::time::Duration;
use serde::Deserialize;
use std::convert::TryFrom;
use thiserror::Error;
use toml::Value;

use awt_simulation::metric::{
    Metric as SimMetric, MetricError as SimMetricError, MetricType as SimMetricType,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize)]
pub enum MetricType {
    ServiceLevel,
    AverageWorkTime,
    AverageSpeedAnswer,
    AverageTimeToAbandon,
    AbandonRate,
    AverageTimeInQueue,
    UtilisationTime,
    AnswerCount,
}

impl core::fmt::Display for MetricType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Deserialize)]
pub struct Metric {
    pub metric: MetricType,
    pub sla: Option<Duration>,
    pub target: Option<Value>,
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum MetricError {
    #[error("SLARequiresWindow: SLA requires a window specified by a sla key")]
    SLARequiresWindow,
    #[error("SLA requires a target in the range of 0.0..1.0. Received {0}")]
    SLAOutsideOfTarget(f64),
    #[error("Target should be a floating point number {0}")]
    TargetFloatingPoint(Value),
    #[error("Target is required for {0}")]
    TargetRequired(MetricType),
    #[error("Conversion error for {0}")]
    ConversionError(#[from] toml::de::Error),
    #[error("Error constructing metric, {0:?}")]
    MetricError(SimMetricError),
    #[error("Metric Not Yet Implemented")]
    NotYetImplemented,
}

impl From<SimMetricError> for MetricError {
    fn from(err: SimMetricError) -> Self {
        Self::MetricError(err)
    }
}

impl TryFrom<&Metric> for SimMetric {
    type Error = MetricError;

    fn try_from(metric: &Metric) -> Result<Self, Self::Error> {
        match metric.metric {
            MetricType::ServiceLevel => {
                let Some(sla) = metric.sla else { return Err(MetricError::SLARequiresWindow) };

                let target = match metric.target.clone() {
                    Some(toml::value::Value::Float(f)) if (0.0..=1.0).contains(&f) => f,
                    Some(toml::value::Value::Float(f)) => {
                        return Err(MetricError::SLAOutsideOfTarget(f))
                    }
                    Some(non_floating) => {
                        return Err(MetricError::TargetFloatingPoint(non_floating))
                    }
                    None => return Err(MetricError::TargetRequired(metric.metric)),
                };

                Ok(Self::with_target_f64(
                    SimMetricType::ServiceLevel(sla),
                    target,
                )?)
            }
            MetricType::AverageWorkTime => {
                let target: Duration = if let Some(value) = metric.target.clone() {
                    value.try_into()?
                } else {
                    Err(MetricError::TargetRequired(metric.metric))?
                };

                Ok(Self::with_target_duration(
                    SimMetricType::AverageWorkTime,
                    target,
                )?)
            }
            MetricType::AverageSpeedAnswer => {
                let target: Duration = if let Some(value) = metric.target.clone() {
                    value.try_into()?
                } else {
                    Err(MetricError::TargetRequired(metric.metric))?
                };

                Ok(Self::with_target_duration(
                    SimMetricType::AverageSpeedAnswer,
                    target,
                )?)
            }
            MetricType::AverageTimeToAbandon => {
                let target: Duration = if let Some(value) = metric.target.clone() {
                    value.try_into()?
                } else {
                    Err(MetricError::TargetRequired(metric.metric))?
                };

                Ok(Self::with_target_duration(
                    SimMetricType::AverageTimeToAbandon,
                    target,
                )?)
            }
            MetricType::AverageTimeInQueue => {
                let target: Duration = if let Some(value) = metric.target.clone() {
                    value.try_into()?
                } else {
                    Err(MetricError::TargetRequired(metric.metric))?
                };

                Ok(Self::with_target_duration(
                    SimMetricType::AverageTimeInQueue,
                    target,
                )?)
            }
            MetricType::AbandonRate => {
                let target: f64 = if let Some(value) = metric.target.clone() {
                    value.try_into()?
                } else {
                    Err(MetricError::TargetRequired(metric.metric))?
                };

                Ok(Self::with_target_f64(SimMetricType::AbandonRate, target)?)
            }
            MetricType::AnswerCount => {
                let target: usize = if let Some(value) = metric.target.clone() {
                    value.try_into()?
                } else {
                    Err(MetricError::TargetRequired(metric.metric))?
                };

                Ok(Self::with_target_usize(SimMetricType::AnswerCount, target)?)
            }

            MetricType::UtilisationTime => Err(MetricError::NotYetImplemented),
        }
    }
}
