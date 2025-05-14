// FIX
// This is a wrong way to import dependencies. It should be refactored.
// So now you have to decide what the best approach is to proceed here. I think
// that you should strive for making the sys
// QUESTION [ ]
// Should you make this work with the
// Where should the system messages be found?

// INFO
// So the idea is that all the functions should be separate. And the endpoints
// should simply call the different functions. What is the difference between
// the orchesatrator functions and handlers? The orchestrator simply has
// `Communication`s, `SchedulingEnvironment` `SystemSolutions` that you can use.
// This is what the orchestrator is. The remaining things should come from the
// handlers. They should provide the information that the orchestrator
// needs to do what it is supposed to do.
// You should make your own data structure here.

// TODO [ ] Make a route and handler for each of these. The Orchestrator
// should be a function parameter to this process.

// #[cfg(test)]
// mod tests
// {
//     use std::collections::HashMap;

//     use chrono::Utc;
//     use shared_types::agents::tactical::Days;
//     use shared_types::agents::tactical::TacticalResources;
//     use shared_types::scheduling_environment::time_environment::day::Day;
//     use shared_types::scheduling_environment::work_order::operation::Work;
//     use shared_types::scheduling_environment::worker_environment::resources::Resources;

//     #[test]
//     fn test_day_serialize()
//     {
//         let mut hash_map_nested = HashMap::<Day, Work>::new();

//         let mut hash_map = HashMap::<Resources, Days>::new();
//         let day = Day::new(0, Utc::now());
//         day.to_string();
//         hash_map_nested.insert(day, Work::from(123.0));

//         hash_map.insert(Resources::MtnMech,
// Days::new(hash_map_nested.clone()));         let tactical_resources =
// TacticalResources::new(hash_map.clone());         serde_json::to_string(&
// tactical_resources).unwrap();     }
// }
