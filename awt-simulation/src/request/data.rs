use crate::request::Status;
use core::time::Duration;

pub struct Data {
    pub id: usize,
    pub status: Status,
    pub wait_time: Option<Duration>,
    pub handle_time: Option<Duration>,
}
