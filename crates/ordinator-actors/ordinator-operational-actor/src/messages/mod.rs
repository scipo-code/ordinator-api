pub mod message_handlers;
pub mod requests;
pub mod responses;
use ordinator_actor_core::RequestMessage;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Deserialize;
use serde::Serialize;

use self::requests::*;
use self::responses::*;

pub type OperationalRequestMessage = RequestMessage<
    OperationalStatusRequest,
    OperationalSchedulingRequest,
    OperationalResourceRequest,
    OperationalTimeRequest,
    OperationalSchedulingEnvironmentCommands,
>;

// You need type safety here I do not see another way around it
//
pub enum ResponseMessage<S, Sc, R, T>
{
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
}

// You should use the module paths in `operational::response::Status`,
// `supervisor::request::Status`. Yes that is the correct approach here.
// I do not think that there is a better way of doing it.
#[derive(Serialize)]
pub enum OperationalResponseMessage
{
    Status(OperationalResponseStatus),
    Scheduling(OperationalSchedulingResponse),
    Resource(OperationalResourceResponse),
    Time(OperationalTimeResponse),
}

#[derive(Serialize)]
pub struct OperationalStatus
{
    objective: f64,
}

// We should stop working on this now. Your primary difficulty here is
// what to do with all these messages. I think that the best thing may be
// to give each Actor a large enum that can handle all the different cases.
//
// What other approaches do I have
// This is so ugly, how could you even get yourself to code this? I think that
// the best thing to do now is take a little break and then continue.
#[derive(Serialize)]
pub enum OperationalResponse
{
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
mod tests
{

    use chrono::DateTime;
    use chrono::NaiveTime;
    use chrono::TimeDelta;
    use ordinator_scheduling_environment::time_environment::TimeInterval;

    #[test]
    fn test_time_interval_contains_1()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T00:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_2()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T20:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(time_interval.contains(&current_time));
    }

    #[test]
    fn test_time_interval_contains_3()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(!time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_4()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time);

        assert!(!time_interval.contains(&current_time));
    }

    #[test]
    fn test_time_interval_duration()
    {
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
