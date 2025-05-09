pub mod message_handlers;
pub mod requests;
pub mod responses;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Deserialize;
use serde::Serialize;

use self::requests::*;
use self::responses::*;

#[derive(Deserialize, Serialize, Debug)]
pub enum OperationalRequest {
    GetIds(Asset),
    AllOperationalStatus(Asset),
    ForOperationalAgent((Asset, String, OperationalRequestMessage)),
}

pub trait Status {}
pub trait Scheduling {}
pub trait Resource {}
pub trait Time {}

pub enum RequestMessage<S, Sc, R, T>
where
    S: Status,
    Sc: Scheduling,
    R: Resource,
    T: Time,
{
    // I am not sure that this is the best approach for making the
    // messages. The messages will always have to work on the
    // `Parameters` and we should use this.
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
}

// You need type safety here I do not see another way around it
//
pub enum ResponseMessage<S, Sc, R, T> {
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
}

// QUESTION TODO [ ]
// Should this be the common message type for the whole system? I
// think that it is a good idea, but before scrutinizing it I think
// that you should strive to make the system as idiot proof as possible
// The idea here is that the Actors should reuse as much as possible.
//
// The Problem here would be if you also needed different kind of functionality
// this goes back to the issue of what to do about the ... I think that the
// best approach here is to make it as. I think that this is the best kind of
// approach to
//
// So each `Actor` should process messages, it needs to be this way as you need
// to have a connection to the `SchedulingEnvironment` and the `Algorithm` and
// the `SystemConfiguration` to make this work. Now the issue is what we should
// do with the Messages. I do not like having
//
// What is it that you do not understand here?
// * You are mixing API types with internal message types. This is causing your
// mind to be unable to
// understand what the path forward is here. Ideally the API types should be
// generating the `OperationalRequestMessage` but this is also not the best
// approach forward here. The question is should we make something that will
// allow us to make something even better.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalRequestMessage {
    Status(OperationalStatusRequest),
    Scheduling(OperationalSchedulingRequest),
    Resource(OperationalResourceRequest),
    Time(OperationalTimeRequest),
}

#[derive(Serialize)]
pub enum OperationalResponseMessage {
    Status(OperationalResponseStatus),
    Scheduling(OperationalSchedulingResponse),
    Resource(OperationalResourceResponse),
    Time(OperationalTimeResponse),
}

#[derive(Serialize)]
pub struct OperationalStatus {
    objective: f64,
}

#[derive(Serialize)]
pub enum OperationalResponse {
    AllOperationalStatus(Vec<OperationalResponseMessage>),
    OperationalIds(Vec<Id>),
    OperationalState(OperationalResponseMessage),
    NoOperationalAgentFound(String),
}

// #[derive(Clone, Deserialize, Serialize, Debug, clap::ValueEnum)]
// pub enum OperationalTarget {
//     #[clap(skip)]
//     Single(OperationalId),
//     All,
// }

// #[derive(Serialize)]
// pub struct OperationalInfeasibleCases {
//     pub operation_overlap: ConstraintState<String>,
// }

// impl OperationalInfeasibleCases {
//     pub fn all_feasible(&self) -> bool {
//         if self.operation_overlap != ConstraintState::Feasible {
//             return false;
//         }
//         true
//     }
// }

// impl Default for OperationalInfeasibleCases {
//     fn default() -> Self {
//         Self {
//             operation_overlap: ConstraintState::Undetermined,
//         }
//     }
// }

#[cfg(test)]
mod tests {

    use chrono::DateTime;
    use chrono::NaiveTime;
    use chrono::TimeDelta;
    use ordinator_scheduling_environment::time_environment::TimeInterval;

    #[test]
    fn test_time_interval_contains_1() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T00:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_2() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T20:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(time_interval.contains(&current_time));
    }

    #[test]
    fn test_time_interval_contains_3() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(!time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_4() {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(!time_interval.contains(&current_time));
    }

    #[test]
    fn test_time_interval_duration() {
        let start = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(7, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(12 * 3600, 0).unwrap()
        );

        let start = NaiveTime::from_hms_opt(2, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(7, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(5 * 3600, 0).unwrap()
        );
        let start = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(1, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(2 * 3600, 0).unwrap()
        );

        let start = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(time_interval.duration(), TimeDelta::new(0, 0).unwrap());
    }
}
