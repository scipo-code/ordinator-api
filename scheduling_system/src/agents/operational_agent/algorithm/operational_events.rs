use chrono::{NaiveTime, TimeDelta};
use shared_types::{
    operational::{operational_response_scheduling::EventType, TimeInterval},
    scheduling_environment::work_order::WorkOrderActivity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationalEvents {
    WrenchTime((TimeInterval, WorkOrderActivity)),
    Break(TimeInterval),
    Toolbox(TimeInterval),
    OffShift(TimeInterval),
    NonProductiveTime(TimeInterval),
    Unavailable(TimeInterval),
}

impl OperationalEvents {
    pub fn time_delta(&self) -> TimeDelta {
        match self {
            Self::WrenchTime((time_interval, _)) => time_interval.duration(),
            Self::Break(time_interval) => time_interval.duration(),
            Self::Toolbox(time_interval) => time_interval.duration(),
            Self::OffShift(time_interval) => time_interval.duration(),
            Self::NonProductiveTime(time_interval) => time_interval.duration(),
            Self::Unavailable(time_interval) => time_interval.duration(),
        }
    }
    pub fn start_time(&self) -> NaiveTime {
        match self {
            Self::WrenchTime((time_interval, _)) => time_interval.start,
            Self::Break(time_interval) => time_interval.start,
            Self::Toolbox(time_interval) => time_interval.start,
            Self::OffShift(time_interval) => time_interval.start,
            Self::NonProductiveTime(time_interval) => time_interval.start,
            Self::Unavailable(time_interval) => time_interval.start,
        }
    }
    pub fn finish_time(&self) -> NaiveTime {
        match self {
            Self::WrenchTime((time_interval, _)) => time_interval.end,
            Self::Break(time_interval) => time_interval.end,
            Self::Toolbox(time_interval) => time_interval.end,
            Self::OffShift(time_interval) => time_interval.end,
            Self::NonProductiveTime(time_interval) => time_interval.end,
            Self::Unavailable(time_interval) => time_interval.end,
        }
    }

    pub fn unavail(&self) -> bool {
        matches!(&self, OperationalEvents::Unavailable(_))
    }

    pub fn is_wrench_time(&self) -> bool {
        matches!(&self, Self::WrenchTime(_))
    }
}

impl Into<EventType> for OperationalEvents {
    fn into(self) -> EventType {
        match self {
            OperationalEvents::WrenchTime(_) => EventType::WrenchTime,
            OperationalEvents::Break(_) => EventType::Break,
            OperationalEvents::Toolbox(_) => EventType::Toolbox,
            OperationalEvents::OffShift(_) => EventType::OffShift,
            OperationalEvents::NonProductiveTime(_) => EventType::NonProductiveTime,
            OperationalEvents::Unavailable(_) => EventType::Unavailable,
        }
    }
}
