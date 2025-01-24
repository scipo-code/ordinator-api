pub mod strategic_parameters;
pub mod assert_functions;

use assert_functions::StrategicAssertions;
use shared_types::scheduling_environment::time_environment::TimeEnvironment;
use strum::IntoEnumIterator;
use crate::agents::traits::LargeNeighborhoodSearch;
use crate::agents::{SharedSolution, StrategicSolution, ArcSwapSharedSolution};
use anyhow::{bail, ensure, Context, Result};
use strategic_parameters::{StrategicClustering, StrategicParameter, StrategicParameterBuilder, StrategicParameters};
use priority_queue::PriorityQueue;
use rand::prelude::SliceRandom;
use shared_types::scheduling_environment::WorkOrders;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::strategic::{OperationalResource, StrategicObjectiveValue, StrategicResources};
use shared_types::strategic::strategic_request_periods_message::StrategicTimeRequest;
use shared_types::strategic::strategic_request_resources_message::StrategicResourceRequest;
use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
use shared_types::strategic::strategic_response_periods::StrategicResponsePeriods;use shared_types::strategic::strategic_response_resources::StrategicResponseResources;
use shared_types::strategic::strategic_response_scheduling::StrategicResponseScheduling;
use shared_types::{Asset, LoadOperation};
use std::any::type_name;
use std::arch::asm;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use itertools::Itertools;

use tracing::{event, instrument, Level};

pub struct StrategicAlgorithm {
    pub priority_queues: PriorityQueues<WorkOrderNumber, u64>,
    pub strategic_parameters: StrategicParameters,
    pub strategic_solution: StrategicSolution,
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: arc_swap::Guard<Arc<SharedSolution>>,
    period_locks: HashSet<Period>,
    pub strategic_periods: Vec<Period>,
}

impl StrategicAlgorithm {
    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    pub fn make_atomic_pointer_swap(&self) {
        // Performance enhancements:
        // * COW:
        //      #[derive(Clone)]
        //      struct SharedSolution<'a> {
        //          tactical: Cow<'a, TacticalSolution>,
        //          // other fields...
        //      }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed.
        //     let shared_solution = Arc::new(SharedSolution {
        //             tactical: self.tactical_solution.clone(),
        //             // Copy over other fields without cloning
        //             ..(**old).clone()
        //         });
        self.arc_swap_shared_solution.0.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.strategic = self.strategic_solution.clone();
            Arc::new(shared_solution)
        });
    
    } 

    pub fn strategic_periods(&self) -> &HashMap<WorkOrderNumber, Option<Period>> {
        &self.strategic_solution.strategic_periods
    }

    pub fn strategic_periods_mut(&mut self) -> &mut HashMap<WorkOrderNumber, Option<Period>> {
        &mut self.strategic_solution.strategic_periods
    }

    pub fn set_periods(&mut self, periods: Vec<Period>) {
        self.strategic_periods = periods;
    }

    pub fn update_the_locked_in_period(&mut self, work_order_number: &WorkOrderNumber, locked_in_period: &Period) -> Result<()> {
        self.strategic_solution.strategic_periods.insert(*work_order_number, Some(locked_in_period.clone()));

        let strategic_parameter = self.strategic_parameters.strategic_work_order_parameters.get_mut(work_order_number).with_context(|| format!("{:?} not found in {}", work_order_number, std::any::type_name::<StrategicParameters>()))?;

        strategic_parameter.excluded_periods.remove(locked_in_period);
        strategic_parameter.locked_in_period = Some(locked_in_period.clone());
        Ok(())
    }
    // pub fn swap_scheduled_work_orders(&mut self, rng: &mut impl rand::Rng) {
        


            
    //         let scheduled_work_orders: Vec<_> = self
    //             .strategic_solution
    //             .strategic_periods
    //             .keys()
    //             .cloned()
    //             .collect();


    //         let randomly_chosen = scheduled_work_orders.choose_multiple(rng, 2).collect::<Vec<_>>();


                

    //         unsafe {
    //             let scheduled_work_order_1 = self.strategic_solution.strategic_periods.get_mut(randomly_chosen[0]).unwrap() as *mut Option<Period>;
    //             let scheduled_work_order_2 = self.strategic_solution.strategic_periods.get_mut(randomly_chosen[1]).unwrap() as *mut Option<Period>;

    //             std::ptr::swap(scheduled_work_order_1, scheduled_work_order_2);

    //             // You cannot do this anymore either. What is the best remedy for this? 
    //             self.update_loadings(randomly_chosen[0], (*scheduled_work_order_1).as_ref().unwrap(), LoadOperation::Sub);
    //             self.update_loadings(randomly_chosen[1], (*scheduled_work_order_2).as_ref().unwrap(), LoadOperation::Sub);
    //             self.update_loadings(randomly_chosen[0], (*scheduled_work_order_1).as_ref().unwrap(), LoadOperation::Add);
    //             self.update_loadings(randomly_chosen[1], (*scheduled_work_order_2).as_ref().unwrap(), LoadOperation::Add);
    //         }




    // }
    pub fn create_strategic_parameters(
        &mut self,
        work_orders: &WorkOrders,
        periods: &[Period],
        asset: &Asset,
    ) {
        for (work_order_number, work_order) in work_orders.inner.iter() {
            if &work_order.functional_location().asset != asset {
                continue;
            }

            let strategic_parameter = StrategicParameterBuilder::new().build_from_work_order(work_order, periods).build();
        
            // What should be coded here? 
            self.load_shared_solution();
            
            self.strategic_parameters.insert_strategic_parameter(*work_order_number, strategic_parameter);
            self.make_atomic_pointer_swap();
        }
    }

    fn strategic_capacity_by_resource(&self, resource: &Resources, period: &Period) -> Result<Work> {
        self
            .strategic_parameters
            .strategic_capacity
            .aggregated_capacity_by_period_and_resource(period, resource)

    }

    fn strategic_loading_by_resource(&self, resource: &Resources, period: &Period) -> Result<Work> {
        self
            .strategic_solution
            .strategic_loadings
            .aggregated_capacity_by_period_and_resource(period, resource)
    }

    #[allow(dead_code)]
    pub fn calculate_utilization(&self) -> Result<Vec<(i32, u64)>> {
        let mut utilization_by_period = Vec::new();

        for period in &self.strategic_periods {
            let mut intermediate_loading: f64 = 0.0;
            let mut intermediate_capacity: f64 = 0.0;
            for resource in Resources::iter() {
                let loading = self.strategic_loading_by_resource(&resource, period)?;
                let capacity = self.strategic_capacity_by_resource(&resource, period)?;

                intermediate_loading += loading.to_f64();
                intermediate_capacity += capacity.to_f64();
                
            }
            let percentage_loading = ((intermediate_loading / intermediate_capacity) * 100.0) as u64;
            utilization_by_period.push((*period.id(), percentage_loading));
        }
        Ok(utilization_by_period)
    }

    fn determine_urgency(&mut self, strategic_objective_value: &mut StrategicObjectiveValue) {
        for (work_order_number, scheduled_period) in &self.strategic_solution.strategic_periods {
            let optimized_period = match scheduled_period {
                Some(optimized_period) => optimized_period.clone(),
                None => {
                    event!(Level::ERROR, "{:?} does not have a scheduled period", work_order_number);
                    panic!("{:?} does not have a scheduled period", work_order_number);
                }
            };

            let work_order_latest_allowed_finish_period =
                &self.strategic_parameters.strategic_work_order_parameters.get(work_order_number).expect("StrategicParameter should always be available for the StrategicSolution").latest_period;

            let non_zero_period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );
    
            let period_penalty = non_zero_period_difference
                * self
                    .strategic_parameters
                    .strategic_work_order_parameters
                    .get(work_order_number)
                    .unwrap()
                    .weight;

            strategic_objective_value.urgency.1 += period_penalty;
        }
    }

    fn determine_clustering(&mut self, strategic_objective_value: &mut StrategicObjectiveValue) {
        for period in &self.strategic_periods {
            // Precompute scheduled work orders for the current period
            let scheduled_work_orders_by_period: Vec<_> = self
                .strategic_solution
                .strategic_periods
                .iter()
                .filter_map(|(won, opt_per)| {
                    if let Some(per) = opt_per {
                        if per == period {
                            return Some(won);
                        }
                    }
                    None
                })
                .collect();

            // Cache references to clustering inner map
            let clustering_inner = &self.strategic_parameters.strategic_clustering.inner;

            for i in 0..scheduled_work_orders_by_period.len() {
                for j in (i + 1)..scheduled_work_orders_by_period.len() {
                    // Retrieve clustering value, handling symmetry
                    let work_order_pair = (
                        *scheduled_work_orders_by_period[i],
                        *scheduled_work_orders_by_period[j],
                    );
                    let reverse_pair = (
                        *scheduled_work_orders_by_period[j],
                        *scheduled_work_orders_by_period[i],
                    );

                    let clustering_value_for_work_order_pair = clustering_inner
                        .get(&work_order_pair)
                        .or_else(|| clustering_inner.get(&reverse_pair))
                        .with_context(|| {
                            format!(
                                "Missing: {} between {:?} and {:?}",
                                std::any::type_name::<StrategicClustering>(),
                                scheduled_work_orders_by_period[i],
                                scheduled_work_orders_by_period[j]
                            )
                        }).unwrap();

                    // Increment the clustering value in the objective
                    strategic_objective_value.clustering_value.1 += *clustering_value_for_work_order_pair;
                }
            }
        }
    }

    // The resource penalty should simply be calculated on the total amount of exceeded hours. I do
    // not see a different way of coding it. It could work on the skill_hours, but that would be
    // needlessly complex in this setting.
    fn determine_resource_penalty(&mut self, strategic_objective_value: &mut StrategicObjectiveValue) {
        for (resource, periods) in &self.resources_capacities().0 {
                let capacity: f64 = periods.iter().map(|ele| ele.1.total_hours.to_f64()).sum();
                let loading: f64 = self
                    .strategic_solution
                    .strategic_loadings
                    .0
                    .get(resource)
                    .unwrap()
                    .iter().map(|ele| ele.1.total_hours.to_f64()).sum();

                if loading - capacity > 0.0 {
                    strategic_objective_value.resource_penalty.1 += (loading - capacity) as u64
                }
            
        }
    }
}

#[derive(Debug)]
pub enum ForcedWorkOrder {
    Locked(WorkOrderNumber),
    FromTactical((WorkOrderNumber, Period)),
}

#[derive(Debug)]
pub enum ScheduleWorkOrder {
    Normal,
    Forced,
    Unschedule,
}

impl StrategicAlgorithm {
    pub fn schedule_forced_work_orders(&mut self) -> Result<()> {
        let tactical_work_orders = &self.loaded_shared_solution.tactical.tactical_scheduled_work_orders;
        let mut work_order_numbers: Vec<ForcedWorkOrder> = vec![];
        // There exists work order parameters that are not in the solution. Is this a problem? I think that it is a problem, but I do not really understand
        // what should be
        for (work_order_number, strategic_parameter) in self.strategic_parameters.strategic_work_order_parameters.iter() {

            let scheduled_period = self
                .strategic_solution
                .strategic_periods
                .get(work_order_number)
                .with_context(|| format!("{:?}\nis not found in the StrategicAlgorithm", work_order_number))?;

            let tactical_work_order = tactical_work_orders
                .0
                .get(work_order_number);


            if scheduled_period == &strategic_parameter.locked_in_period {
                continue
            }

            if strategic_parameter.locked_in_period.is_some() {
                work_order_numbers.push(ForcedWorkOrder::Locked(*work_order_number));
            } else if tactical_work_order.is_some() && tactical_work_order.unwrap().is_tactical() {
                let first_day = tactical_work_order.unwrap()
                    .tactical_operations()?
                    .0
                    .iter()
                    .min_by(|ele1 , ele2| {
                        ele1.1.scheduled[0].0.date().date_naive().cmp(&ele2.1.scheduled[0].0.date().date_naive())
                    }).unwrap()
                    .1
                    .scheduled[0].0.date().date_naive();

                let tactical_period = self
                    .strategic_periods
                    .iter()
                    .find(|per| per.contains_date(first_day))
                    .expect("This result would come directly from the tactical agent. It should always find a Period in the Vec<Period>");

                work_order_numbers.push(ForcedWorkOrder::FromTactical((*work_order_number, tactical_period.clone())));
            } 
        }

        for forced_work_order_numbers in work_order_numbers {
            self.schedule_forced_work_order(&forced_work_order_numbers).with_context(|| format!("{:?} could not be force scheduled", forced_work_order_numbers))?;
        }
        Ok(())
    }

    pub fn schedule_normal_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>> {
        let strategic_parameter = self
            .strategic_parameters
            .strategic_work_order_parameters
            .get(&work_order_number)
            .unwrap()
            .clone();

        let mut resource_use = StrategicResources::default();

        if period != self.periods().last().unwrap() {
            if strategic_parameter.excluded_periods().contains(period) {
                return Ok(Some(work_order_number));
            }

            if self.period_locks.contains(period) {
                return Ok(Some(work_order_number));
            }

            let work_load = &self.strategic_parameters.strategic_work_order_parameters.get(&work_order_number).unwrap().work_load;


            event!(Level::INFO, strategic_work_order = ?work_order_number);
            let resource_use_option = self
                .determine_best_permutation( work_load.clone(), period, ScheduleWorkOrder::Normal)
                .with_context(|| format!("{:?}\n for period\n{:#?}\ncould not be {:?}", work_order_number, period, ScheduleWorkOrder::Normal))?;

            match resource_use_option {
                Some(resource_use_inner) => resource_use = resource_use_inner,
                None => return Ok(Some(work_order_number)),
            }
        }

        let previous_period = self
            .strategic_periods_mut()
            .insert(work_order_number, Some(period.clone()));

        ensure!(previous_period.as_ref().unwrap().is_none(), "Previous period: {:#?}. New period: {:#?}", &previous_period, period);
        
        self.update_loadings(resource_use, LoadOperation::Add);
        Ok(None)
    }

    pub fn schedule_forced_work_order(&mut self, force_schedule_work_order: &ForcedWorkOrder) -> Result<()> {
        let work_order_number = match force_schedule_work_order {
            ForcedWorkOrder::Locked(work_order_number) => work_order_number,
            ForcedWorkOrder::FromTactical((work_order_number, _)) => work_order_number,
        };
        
        if let Some(work_order_number) = self.is_scheduled(work_order_number) {
            self.unschedule(*work_order_number)
                .with_context(|| format!("{:?}\n{}\n{}", force_schedule_work_order, file!(), line!()))?
        }

        let locked_in_period = match &force_schedule_work_order {
            ForcedWorkOrder::Locked(work_order_number) => self
                .strategic_parameters
                .get_locked_in_period(work_order_number).clone(),
            ForcedWorkOrder::FromTactical((_, period)) => period.clone(),
        };

        // Should the update loadings also be included here? I do not think that is a good idea.
        // What other things could we do?
        self.update_the_locked_in_period(work_order_number, &locked_in_period.clone())
            .with_context(|| format!("Could not fully update {:#?} in {}", force_schedule_work_order, &locked_in_period))?;

        let work_load = self.strategic_parameters.strategic_work_order_parameters.get(work_order_number).unwrap().work_load.clone();
        let strategic_resources = self
            .determine_best_permutation(work_load, &locked_in_period, ScheduleWorkOrder::Forced)
            .with_context(|| format!("{:?}\nin period {:#?}\ncould not be\n{:?}", force_schedule_work_order, locked_in_period, ScheduleWorkOrder::Forced))?
            .expect("It should always be possible to determine a resource permutation for a forced work order");

        self.update_loadings(strategic_resources, LoadOperation::Add);
        Ok(())
    }

    pub fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: usize,
        rng: &mut impl rand::Rng,
    ) -> Result<()> {
        let strategic_periods = &self.strategic_solution.strategic_periods;

        let strategic_parameters = &self.strategic_parameters.strategic_work_order_parameters;

        let mut filtered_keys: Vec<_> = strategic_periods
            .iter()
            .filter(|(won, _)| strategic_parameters.get(won).unwrap().locked_in_period.is_none())
            .map(|(&won, _)| won)
            .collect();

        filtered_keys.sort();

        let sampled_work_order_keys = filtered_keys
            .choose_multiple(rng, number_of_work_orders)
            .collect::<Vec<_>>()
            .clone();

        // assert!(self.strategic_solution.scheduled_periods.values().all(|per| per.is_some()));
        for work_order_number in sampled_work_order_keys {
            self.unschedule(*work_order_number).with_context(|| format!("{:?} should always be present", work_order_number))?;
            self.populate_priority_queues();
        }
        Ok(())
    }

    fn is_scheduled<'a>(&'a self, work_order_number: &'a WorkOrderNumber) -> Option<&'a WorkOrderNumber> {
        self.strategic_periods()
            .get(work_order_number)
            .and_then(|scheduled_period| {
                scheduled_period
                    .as_ref()
                    .map(|_| work_order_number)
            })
    }

    /// This function updates the StrategicResources based on the a provided loading.
    pub fn update_loadings(&mut self, strategic_resources: StrategicResources, load_operation: LoadOperation) {
        // How should the change be handled in this function? The most important thing here is to make the function work correctly
        // on the new resource type. This will be difficult as we cannot make the
        // FIX This loading function is not correct. How should we change it so that it will be able to function correctly without
        // calling the permutation loop? We cannot, it would not make sense for this kind of function. I believe that the best
        // decision here is to make a function that lets us manually update and change a value. 
        for (period, operational_resources) in strategic_resources.0 {
            for (operational_id, loading) in operational_resources {
                match load_operation {
                    LoadOperation::Add => {
                        let strategic_loading = self
                            .strategic_solution
                            .strategic_loadings
                            .0
                            .get_mut(&period)
                            .unwrap()
                            .get_mut(&operational_id)
                            .unwrap();

                        strategic_loading.total_hours += loading.total_hours;
                        strategic_loading.skill_hours.iter_mut().for_each(|(res, wor)| {
                              *wor += loading.skill_hours.get(res).unwrap();  
                        });

                    },
                    LoadOperation::Sub => {
                        let strategic_loading = self
                            .strategic_solution
                            .strategic_loadings
                            .0
                            .get_mut(&period)
                            .unwrap()
                            .get_mut(&operational_id)
                            .unwrap();

                        strategic_loading.total_hours -= loading.total_hours;
                        strategic_loading.skill_hours.iter_mut().for_each(|(res, wor)| {
                              *wor -= loading.skill_hours.get(res).unwrap();  
                        });

                    },
                }
            }
        }
    }


    /// This function is created to find the best permutation of all the work order assignments to all technicians.
    /// This function has two purposes:
    /// * Determine if there is a feasible permutaion
    /// * If true, return the loading that should be put into the StrategicSolution::strategic_loadings.
    pub fn determine_best_permutation(&self, work_load: HashMap<Resources, Work>, period: &Period, schedule: ScheduleWorkOrder) -> Result<Option<StrategicResources>> {

        // How do we take this into account? For now the tacitcal is simply an aggregated version of the strategic, which of course is not
        // the way to approach this problem. 
        let mut best_total_excess = Work::from(0.0); 
        let mut best_work_order_resource_loadings = StrategicResources::default();

        // We want to find the difference between the two resources, as this is the amount of
        // capacity that we can effectively schedule with. 
        let capacity_resources = self.strategic_parameters.strategic_capacity.0.get(period).unwrap();

        // If this is a normal scheduling operation we make sure that there
        // actually is the required skills available in the technicians. 
        if matches!(schedule, ScheduleWorkOrder::Normal) && !work_load
            .keys()
            .collect::<HashSet<&Resources>>()
            .is_subset(&capacity_resources.values().flat_map(|ele| ele.skill_hours.keys()).collect::<HashSet<&Resources>>()) {
            
            return Ok(None)
        }

        let loading_resources = self.strategic_solution.strategic_loadings.0.get(period).unwrap();
        
        let difference_resources = {
            let mut difference_resources = HashMap::new();

            for capacity in capacity_resources {
                
                let loading = loading_resources.get(capacity.0).unwrap();

                // Okay so the total amount of resources have to be reduced based on the
                // TacticalResource. What should we do to correct for this? Later on
                // we could make the code work on the individual technician when he
                // is implementing the multi-skill as well.
                // FIX: Fix this after adjusting the TacticalAlgorithm to account for
                // multiskilled personal.
                // let loading_coming_from_the_tactical_agent = self
                //     .loaded_shared_solution
                //     .tactical
                //     .tactical_loadings
                //     .determine_period_load(resource, period)
                //     .unwrap_or_default();
                let total_hours = capacity.1.total_hours - loading.total_hours;
                let skill_hours = capacity
                    .1
                    .skill_hours
                    .clone()
                    .iter()
                    .zip(loading.skill_hours.iter())
                    .map(|(cap, loa)| (cap.0.clone(), (cap.1 - loa.1)))
                    .collect();

                let operational_resource = OperationalResource::new(
                    capacity.0.to_string(),
                    total_hours,
                    skill_hours
                );

                difference_resources.insert(capacity.0.clone(), operational_resource);
            }
            difference_resources
        };

        let permutation_length = difference_resources.len();
        let mut strategic_count_technician_permution = 0; 
        for mut technician_permutation in difference_resources.clone().into_iter().permutations(permutation_length) {
            strategic_count_technician_permution += 1;
            event!(Level::INFO, strategic_count_technician_permution = strategic_count_technician_permution);


            // If a work_load_permutation iteration is run to completion we accept that solution. 
            let mut strategic_count_work_load_permutaion = 0; 
            for mut work_load_permutation in work_load.clone().into_iter().permutations(work_load.len()) {
                strategic_count_work_load_permutaion += 1;
                event!(Level::INFO, strategic_count_work_load_permutaion = strategic_count_work_load_permutaion);

                // So you create a 'StrategicResource' here and you create a stabil API for that type!
                // Do not focus on performance here. Getting the correct state changes is the most
                // important thing.
                let mut work_order_resource_loadings = StrategicResources::default();

                match schedule {
                    ScheduleWorkOrder::Normal => {
                        // This is what we need! Each technician object now has its own type.
                        // So the permutation creates new instances, that means that we should
                        let mut strategic_count_operation_load = 0; 
                        // only focus on making the calculations.
                        for operation_load in &mut work_load_permutation {
                            strategic_count_operation_load += 1;
                            event!(Level::INFO, strategic_count_operation_load = strategic_count_operation_load);

                            for technician in technician_permutation.clone().iter_mut() {
                                // If the technician does not have the skill we simply skip over that
                                // technician and continue the search. 
                                if !technician.1.skill_hours.keys().contains(&operation_load.0) {
                                    continue;
                                }

                                // You have to know what resources are in here to make it work correctly. I think
                                // that the best approach is to create something that will allow me to 
                                // TODO. What should be done here? We need to update the code so that the correct
                                // amount of capacity is spread out on the technicians.
                                if operation_load.1 <= technician.1.total_hours {
                                    technician.1.skill_hours.iter_mut().for_each(|(_, wor)| *wor -= operation_load.1);  
                                    technician.1.total_hours -= operation_load.1;
                                    // This is the main API function for the Resources. 
                                    work_order_resource_loadings.update_load(period, operation_load.1, technician, LoadOperation::Add).unwrap();
                                    operation_load.1 = Work::from(0.0);
                                    break;
                                } else {
                                    technician.1.skill_hours.iter_mut().for_each(|(_res, wor)| *wor = Work::from(0.0));
                                    technician.1.total_hours = Work::from(0.0);
                                    operation_load.1 -= technician.1.total_hours;
                                    work_order_resource_loadings.update_load(period, technician.1.total_hours, technician, LoadOperation::Add).unwrap();
                                }
                            }
                            // If you have run through all the technicians and the
                            // operation_load is not equal to zero we should break
                            // because then there is no way for that permutation to
                            // satisfy the constraint. 
                            if operation_load.1 != Work::from(0.0) {
                                break;
                            }
                        }
                    },
                    ScheduleWorkOrder::Forced => {
                        // Here we should count the total amount of hours and simply spread them as
                        // evenly as possible out across the agents.

                        // TODO Count all total hours for all of the resources. And then aggregate
                        // them by subtracting the work load, after which you update all the individual
                        // technicians. This function should be completely seperate.
                        
                        // We want to subtract a work order load permutation from a technician permutation
                        // and find the one with the lowest resource penalty. Good. I do not see a good
                        // approach for coding this. So still go over all the permutations and then
                        // combine them at last. 
                        // WARN: We know that the work_load resources are unique.
                        for work in work_load_permutation.iter_mut() {
                            // Count the number of resources satisfying the skill
                            // and split the amount equaly among the technicians.
                            // Good! This is the right way to approach it.
                            // FIX: Here is the issue you are filtering on the qualified technician. But there might not be any that are qualified
                            // to handle the job. What should you do instead? 
                            //
                            // QUESTION:
                            // There are basically two approaches here:
                            // * Split the work on all the current technicians
                            // * Put the work onto a new technician
                            //
                            // What is the best approach forward here? I think that the
                            // best approach is to create
                            //
                            // The Scheduler should create a Resource if a VEN-MECH or something like that
                            // is coming out on the platform. If he does not it only make sense that the
                            // hours be split evenly among the different 
                            let mut qualified_technicians = technician_permutation
                                .iter_mut()
                                .filter(|tec| {
                                    tec.1.skill_hours.keys().contains(&work.0)
                                }).collect::<Vec<_>>();

                            // If there are no qualified technicians. All technicians should be assumed to be responsible? Yes let us just do that!
                            if qualified_technicians.is_empty() {
                                qualified_technicians = technician_permutation.iter_mut().collect();
                            }

                            let work_load_by_resources_by_technician = Work::from(work.1.to_f64() / qualified_technicians.len() as f64);

                            for technician in qualified_technicians.iter_mut() {
                                technician.1.total_hours -= work_load_by_resources_by_technician;
                                technician.1.skill_hours.iter_mut().for_each(|(_, wor)| *wor -= work_load_by_resources_by_technician);

                                work_order_resource_loadings.update_load(period, technician.1.total_hours, technician, LoadOperation::Add).unwrap();
                            }
                        }

                        // Count all the negative in the difference resources and update the current best if we found
                        // an assignment with less penalty.
                        let total_excess = technician_permutation
                            .iter()
                            .map(|(_, or)| {
                                std::cmp::min(Work::from(0.0), or.total_hours)
                            })
                            .reduce(|acc, wor| acc + wor)
                            .expect("The Technician Permutation here should never be empty");

                        if total_excess < best_total_excess {
                            // Save the work_order_resource_loadings
                            best_work_order_resource_loadings = work_order_resource_loadings.clone();
                            best_total_excess = total_excess;
                        }
                    },
                    ScheduleWorkOrder::Unschedule => {
                        
                        // Unscheduling is easier but we need to be clear on how we should handle
                        // the different resources. QUESTION: How should we subtract the different
                        // resources. The period is given and the question is how to subtract the
                        // correct resources from the individual technician. It should be the
                        // reverse of what we are doing in the other case. 
                        // QUESTION: Do we want to use the different permutations? I think that
                        // we do not. It should rely on the... The difference says how much
                        // resource there is left. We also need to loadings to subtract the
                        // resource from the work_load. We only need the loading_resources
                        // it should 

                        // We should subtract until there are no more resources left
                        let mut work_order_resource_loadings = StrategicResources::default();

                        let mut technician_loadings = loading_resources.clone();

                        for work in work_load_permutation.iter_mut() {
                            // Here you have to subtract the technician_permutation.
                            for technician_permutation in &mut technician_permutation {
                                let technician = technician_loadings.get_mut(&technician_permutation.0).unwrap();

                                if !technician.skill_hours.contains_key(&work.0) {
                                    continue
                                }

                                if  technician.total_hours >= work.1 {
                                    technician.total_hours -= work.1;
                                    technician.skill_hours.iter_mut().for_each(|ele| *ele.1 -= work.1);
                                    work_order_resource_loadings.update_load(period, work.1, &(technician_permutation.0.clone(), technician.clone()), LoadOperation::Sub).expect("Resource subtraction should always be possible.");
                                    work.1 = Work::from(0.0);

                                    // If all the resource is removed from the operation_work, we should break and move on to the next operation.
                                    break;
                                } else {
                                    work.1 -= technician.total_hours;
                                    work_order_resource_loadings.update_load(period, technician.total_hours, &(technician_permutation.0.clone(), technician.clone()), LoadOperation::Sub).expect("Resource subtraction should always be possible.");
                                    technician.total_hours = Work::from(0.0); 
                                    technician.skill_hours.iter_mut().for_each(|ele| *ele.1 = Work::from(0.0));

                                    // If the select technician does not have enough hours left we should subtract we
                                    // should move on to the next technician.
                                    continue;                                    
                                }
                            }
                        }
                        if work_load_permutation.iter().all(|res_wor| res_wor.1 == Work::from(0.0)) {
                            return Ok(Some(work_order_resource_loadings));
                        }
                    },
                }

                match schedule {
                    ScheduleWorkOrder::Normal => {
                        if work_load_permutation.into_iter().all(|(_, wor)| wor == Work::from(0.0)) {
                            // If a feasible assignment of work orders were found return the StratgicResources
                            // that should update the StrategicAlgorithm Loadings.
                            return Ok(Some(work_order_resource_loadings))
                        }
                    },
                    ScheduleWorkOrder::Forced => {
                        ensure!(best_work_order_resource_loadings.0.get(period).unwrap().values().flat_map(|ele| ele.skill_hours.keys()).collect::<HashSet<_>>() ==
                            work_load.keys().collect::<HashSet<_>>());
                        return Ok(Some(best_work_order_resource_loadings))
                        
                    },
                    // It is always possible to unschedule work, so therefore we can simply
                    // 
                    ScheduleWorkOrder::Unschedule => {
                        unsafe { asm!("int3") }
                        ensure!(work_order_resource_loadings
                            .0
                            .get(period).with_context(|| format!("{:#?}\nnot present. This probably means that nothing was {:?}\n{}\n{}", period, ScheduleWorkOrder::Unschedule, file!(), line!()))?
                            .iter()
                            .fold(Work::from(0.0), |acc, ele| acc + ele.1.total_hours)
                            == work_load.values().fold(Work::from(0.0), |acc, ele| acc + *ele), format!("{:?} {:?}", work_load, loading_resources));
                        return Ok(Some(work_order_resource_loadings));
                    },
                
                }
            }
        }
        match schedule {
            ScheduleWorkOrder::Normal => Ok(None),
            ScheduleWorkOrder::Forced => Ok(Some(best_work_order_resource_loadings)),
            ScheduleWorkOrder::Unschedule => {
                unsafe { asm!("int3");}
                unreachable!("Unscheduling work order should always be possible");
            }
        }
    }
}



pub fn calculate_period_difference(scheduled_period: Period, latest_period: &Period) -> u64 {
    let scheduled_period_date = scheduled_period.end_date().to_owned();
    let latest_date = latest_period.end_date();
    let duration = scheduled_period_date.signed_duration_since(latest_date);
    let days = duration.num_days();
    std::cmp::max(days / 7, 0) as u64
}

impl LargeNeighborhoodSearch for StrategicAlgorithm {
    type BetterSolution = ();
    type SchedulingRequest = StrategicSchedulingRequest;
    type SchedulingResponse = StrategicResponseScheduling;
    type ResourceRequest = StrategicResourceRequest;
    type ResourceResponse = StrategicResponseResources;
    type TimeRequest = StrategicTimeRequest;
    type TimeResponse = StrategicResponsePeriods;
    type SchedulingUnit = WorkOrderNumber;
    
    fn calculate_objective_value(&mut self) -> Self::BetterSolution {

        let mut strategic_objective_value = StrategicObjectiveValue::new((1, 0), (1_000_000_000, 0), (1_000, 0)); 

        self.determine_urgency(&mut strategic_objective_value);
        
        self.determine_resource_penalty(&mut strategic_objective_value);

        self.determine_clustering(&mut strategic_objective_value);

        strategic_objective_value.aggregate_objectives();

        self.strategic_solution.objective_value = strategic_objective_value;
    }

    #[instrument(level = "trace", skip_all)]
    fn schedule(&mut self) -> Result<()> {
        while !self.priority_queues.normal.is_empty() {
            for period in self.strategic_periods.clone() {
                let (work_order_number, weight) = match self.priority_queues.normal.pop() {
                    Some((work_order_number, weight)) => (work_order_number, weight),
                    None => {
                        break;
                    }
                };                

                let inf_work_order_number = self
                    .schedule_normal_work_order(work_order_number, &period)
                    .with_context(|| format!("{:?} could not be scheduled normally", work_order_number))?;

                if let Some(work_order_number) = inf_work_order_number {
                    self.priority_queues.normal.push(work_order_number, weight);
                }
            }
        }
        Ok(())
    }
    
    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) -> Result<()> {
        let unschedule_from_period= self
            .strategic_periods_mut()
            .get_mut(&work_order_number)
            .with_context(|| format!("{:?}: was not present in the strategic periods", work_order_number))?;

        match unschedule_from_period.take() {
            Some(unschedule_from_period) => {
                let work_load = self.strategic_parameters.strategic_work_order_parameters.get(&work_order_number).unwrap().work_load.clone();
                let strategic_resources = self
                    .determine_best_permutation(work_load, &unschedule_from_period, ScheduleWorkOrder::Unschedule)
                    .with_context(|| format!("{:?}\nin period {:#?}\nfor {:?}", work_order_number, unschedule_from_period, ScheduleWorkOrder::Unschedule))?
                    .context("Determining the StrategicResources associated with a unscheduling operation should always be possible")?;

                self.update_loadings(strategic_resources, LoadOperation::Sub);
            }
            None => bail!("The strategic {:?} was not scheduled but StrategicAlgorithm.unschedule() was called on it.", work_order_number),
        }
        Ok(())
    }

    fn update_resources_state(
        &mut self,
        strategic_resources_message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse> 
    {
        match strategic_resources_message {
            StrategicResourceRequest::GetLoadings {
                periods_end: _,
                select_resources: _,
            } => {
                let loading = self.resources_loadings();

                let strategic_response_resources = StrategicResponseResources::LoadingAndCapacities(loading.clone());
                Ok(strategic_response_resources)
            }
            StrategicResourceRequest::GetCapacities { periods_end: _, select_resources: _ } => 
            {         
                let capacities = self.resources_capacities();

                let strategic_response_resources = StrategicResponseResources::LoadingAndCapacities(capacities.clone());
                Ok(strategic_response_resources)
            }
            StrategicResourceRequest::GetPercentageLoadings { periods_end:_, resources: _ } => {
                let capacities = self.resources_capacities();
                let loadings = self.resources_loadings();

                StrategicAlgorithm::assert_that_capacity_is_respected(loadings, capacities).context("Loadings exceed the capacities")?;
                Ok(StrategicResponseResources::Percentage(capacities.clone(), loadings.clone()))
            }
        }
    }

    #[allow(dead_code)]
    fn update_time_state(&mut self, _time_message: Self::TimeRequest) -> Result<Self::TimeResponse> 
        { todo!() }

    #[instrument(level = "info", skip_all)]
    fn update_scheduling_state(
        &mut self,
        strategic_scheduling_message: StrategicSchedulingRequest,
    ) -> Result<Self::SchedulingResponse>
{
        match strategic_scheduling_message {
            StrategicSchedulingRequest::Schedule(schedule_work_order) => {
                let period = self
                    .strategic_periods
                    .iter()
                    .find(|period| {
                        period.period_string() == schedule_work_order.period_string()
                    })
                    .cloned()
                    .with_context(|| format!("period: {:?} does not exist", schedule_work_order.period_string()))?;
    

                let mut number_of_work_orders = 0;
                for work_order_number in schedule_work_order.work_order_number {
                    let strategic_parameter = self.strategic_parameters.strategic_work_order_parameters.get_mut(&work_order_number).unwrap();
                    if strategic_parameter.excluded_periods.contains(&period) {
                        strategic_parameter.excluded_periods.remove(&period);
                    } 
                    self.strategic_parameters
                        .set_locked_in_period(work_order_number, period.clone()).context("could not set locked in period")?;
                    number_of_work_orders += 1;
                }

                Ok(StrategicResponseScheduling::new(number_of_work_orders, period))
                
            }
            StrategicSchedulingRequest::ExcludeFromPeriod(exclude_from_period) => {

                let period = self.strategic_periods.iter().find(|period| {
                    period.period_string() == exclude_from_period.period_string().clone()
                }).with_context(|| format!("{} was not found in the {}", exclude_from_period.period_string, type_name::<TimeEnvironment>()))?;
                
                let mut number_of_work_orders = 0;
                for work_order_number in exclude_from_period.work_order_number {
                    let strategic_parameter = 
                        self
                            .strategic_parameters
                            .strategic_work_order_parameters
                            .get_mut(&work_order_number)
                            .with_context(|| format!("The {:?} was not found in the {:#?}. The {:#?} should have been initialized at creation.", work_order_number, type_name::<StrategicParameter>(), type_name::<StrategicParameter>()))?;
            
                    assert!(!strategic_parameter.excluded_periods.contains(self.strategic_solution.strategic_periods.get(&work_order_number).as_ref().unwrap().as_ref().unwrap()));
                    strategic_parameter
                        .excluded_periods
                        .insert(period.clone());

                    // assert!(!strategic_parameter.excluded_periods.contains(self.strategic_solution.strategic_periods.get(&work_order_number).as_ref().unwrap().as_ref().unwrap()));

                    if let Some(locked_in_period) = &strategic_parameter.locked_in_period {
                        if strategic_parameter.excluded_periods.contains(locked_in_period) {
                            strategic_parameter.locked_in_period = None;
                            event!(Level::INFO, "{:?} has been excluded from period {} and the locked in period has been removed", work_order_number, period.period_string());
                        }
                    }

                    let last_period = self.strategic_periods.iter().last().cloned();
                    self.strategic_solution.strategic_periods.insert(work_order_number, last_period);

                    assert!(!strategic_parameter.excluded_periods.contains(self.strategic_solution.strategic_periods.get(&work_order_number).as_ref().unwrap().as_ref().unwrap()));
                    number_of_work_orders += 1;
                }

                Ok(StrategicResponseScheduling::new(number_of_work_orders, period.clone()))
            }
        }
    }
}

impl StrategicAlgorithm {
    pub fn new(
        priority_queues: PriorityQueues<WorkOrderNumber, u64>,
        strategic_parameters: StrategicParameters,
        strategic_tactical_solution_arc_swap: Arc<ArcSwapSharedSolution>,
        period_locks: HashSet<Period>,
        periods: Vec<Period>,
    ) -> Self {

        let loaded_shared_solution = strategic_tactical_solution_arc_swap.0.load();

        StrategicAlgorithm {
            priority_queues,
            strategic_parameters,
            strategic_solution: StrategicSolution::default() ,
            arc_swap_shared_solution: strategic_tactical_solution_arc_swap,
            loaded_shared_solution,
            strategic_periods: periods,
            period_locks,
            
        }
    }

    pub fn resources_loadings(&self) -> &StrategicResources {
        &self.strategic_solution.strategic_loadings
    }


    pub fn resources_capacities(&self) -> &StrategicResources {
        &self.strategic_parameters.strategic_capacity
    }


    pub fn periods(&self) -> &Vec<Period> {
        &self.strategic_periods
    }

    pub fn populate_priority_queues(&mut self) {
        for work_order_number in self.strategic_solution.strategic_periods.keys() {
            event!(Level::TRACE, "Work order {:?} has been added to the normal queue", work_order_number);


            if self.strategic_periods().get(work_order_number).unwrap().is_none() {
                let strategic_work_order_weight = self.strategic_parameters.strategic_work_order_parameters.get(work_order_number).expect("The StrategicParameter should always be available for the StrategicSolution").weight;
                self.priority_queues
                    .normal
                    .push(*work_order_number, strategic_work_order_weight);
            }
        }
    }

}


#[derive(Debug, Clone)]
pub struct PriorityQueues<T, P>
where
    T: Hash + Eq,
    P: Ord,
{
    pub normal: PriorityQueue<T, P>,
}

impl PriorityQueues<WorkOrderNumber, u64> {
    pub fn new() -> Self {
        Self {
            normal: PriorityQueue::<WorkOrderNumber, u64>::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arc_swap::ArcSwap;
    use strategic_parameters::{StrategicClustering, StrategicParameter};
    use rand::{rngs::StdRng, SeedableRng};

    use shared_types::scheduling_environment::worker_environment::resources::Resources;

    use crate::agents::{strategic_agent::algorithm::{
            PriorityQueues, StrategicAlgorithm, StrategicParameters
        }, TacticalSolutionBuilder, WhereIsWorkOrder};

    use std::{collections::HashMap, str::FromStr};


    impl StrategicParameter {
        pub fn new(
            locked_in_period: Option<Period>,
            excluded_periods: HashSet<Period>,
            latest_period: Period,
            weight: u64,
            work_load: HashMap<Resources, Work>,
        ) -> Self {
            Self {
                locked_in_period,
                excluded_periods,
                latest_period,
                weight,
                work_load,
            }
        }
    }



    #[test]
    fn test_unschedule_random_work_orders() {
        let mut work_load_1 = HashMap::new();
        let mut work_load_2 = HashMap::new();
        let mut work_load_3 = HashMap::new();

        work_load_1.insert(Resources::MtnMech,Work::from( 10.0));
        work_load_1.insert(Resources::MtnElec,Work::from( 10.0));
        work_load_1.insert(Resources::Prodtech,Work::from( 10.0));

        work_load_2.insert(Resources::MtnMech,Work::from( 20.0));
        work_load_2.insert(Resources::MtnElec,Work::from( 20.0));
        work_load_2.insert(Resources::Prodtech,Work::from( 20.0));

        work_load_3.insert(Resources::MtnMech,Work::from( 30.0));
        work_load_3.insert(Resources::MtnElec,Work::from( 30.0));
        work_load_3.insert(Resources::Prodtech,Work::from( 30.0));

        let mut strategic_parameters = StrategicParameters::new(HashMap::new(), StrategicResources::default(), StrategicClustering::default());

        let strategic_parameter_1 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_1,
        );

        let strategic_parameter_2 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_2,
        );

        let strategic_parameter_3 = StrategicParameter::new(
            None,
            HashSet::new(),
Period::from_str("2023-W49-50").unwrap(),
            1000,
            work_load_3,
        );

        let work_order_number_1 = WorkOrderNumber(2200000001);
        let work_order_number_2 = WorkOrderNumber(2200000002);
        let work_order_number_3 = WorkOrderNumber(2200000003);

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number_1, strategic_parameter_1);

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number_2, strategic_parameter_2);

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number_3, strategic_parameter_3);

        let periods: Vec<Period> = vec![
            Period::from_str("2023-W47-48").unwrap(),
            Period::from_str("2023-W49-50").unwrap(),
        ];

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            periods.clone(),
        );

        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number_1, Some(periods[0].clone()));
        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number_2, Some(periods[1].clone()));
        strategic_algorithm.strategic_solution.strategic_periods.insert(work_order_number_3, Some(periods[1].clone()));

        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];

        let mut rng = StdRng::from_seed(seed);

        strategic_algorithm.unschedule_random_work_orders(2, &mut rng).expect("It should always be possible to unschedule random work orders in the strategic agent");

        assert_eq!(
            *strategic_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000001))
                .unwrap(),
            Some(Period::from_str("2023-W47-48").unwrap())
        );

        assert_eq!(
            *strategic_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000002))
                .unwrap(),
            None
        );

        assert_eq!(

            *strategic_algorithm
                .strategic_periods()
                .get(&WorkOrderNumber(2200000003))
                .unwrap(),
            None
        );
    }

    #[test]
    fn test_calculate_period_difference_1() {
        let scheduled_period = Period::from_str("2023-W47-48");
        let latest_period = Period::from_str("2023-W49-50");

        let difference = calculate_period_difference(scheduled_period.unwrap(), &latest_period.unwrap());

        assert_eq!(difference, 0);
    }
    #[test]
    fn test_calculate_period_difference_2() {
        let period_1 = Period::from_str("2023-W47-48");
        let period_2 = Period::from_str("2023-W45-46");

        let difference = calculate_period_difference(period_1.unwrap(), &period_2.unwrap());

        assert_eq!(difference, 2);
    }


    #[test]
    fn test_choose_multiple() {
        for _ in 0..19 {
            let seed: [u8; 32] = [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];

            let mut rng = StdRng::from_seed(seed);

            assert_eq!(
                [1, 2, 3].choose_multiple(&mut rng, 2).collect::<Vec<_>>(),
                [&3, &2]
            );
        }
    }

    #[test]
    fn test_unschedule_work_order_none_in_scheduled_period() {
        let work_order_number = WorkOrderNumber(2100000001);
        let mut strategic_parameters = StrategicParameters::new(HashMap::new(), StrategicResources::default(), StrategicClustering::default());

        let strategic_parameter = StrategicParameter::new(
            None,
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::new(),
        );

        strategic_parameters
            .strategic_work_order_parameters
            .insert(work_order_number, strategic_parameter);

        let tactical_solution_builder = TacticalSolutionBuilder::new();
       
        let mut tactical_days = HashMap::new();
        tactical_days.insert(work_order_number, WhereIsWorkOrder::NotScheduled);
        
        let tactical_solution = tactical_solution_builder.with_tactical_days(tactical_days).build();

        let shared_solution = SharedSolution {
            tactical: tactical_solution,
            ..SharedSolution::default()
        };

        let arc_swap_shared_solution = ArcSwapSharedSolution(ArcSwap::from_pointee(shared_solution));

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            arc_swap_shared_solution.into(),
            HashSet::new(),
            vec![Period::from_str("2023-W47-48").unwrap()],
        );

        strategic_algorithm
            .strategic_solution
            .strategic_periods
            .insert(work_order_number, Some(Period::from_str("2024-W41-42").unwrap()));

        strategic_algorithm.schedule_forced_work_orders().unwrap();

        strategic_algorithm.unschedule(work_order_number).unwrap();
        assert_eq!(
            *strategic_algorithm
                .strategic_periods()
                .get(&work_order_number)
                .unwrap(),
            None
        );
    }

    #[test]
    fn test_period_clone_equality() {
        let period_1 = Period::from_str("2023-W47-48").unwrap();
        let period_2 = Period::from_str("2023-W47-48").unwrap();

        assert_eq!(period_1, period_2);
        assert_eq!(period_1, period_1.clone());
    }

    impl StrategicAlgorithm {
        pub fn set_strategic_parameter(
            &mut self,
            work_order_number: WorkOrderNumber,
            optimized_work_order: StrategicParameter,
        ) {
            self.strategic_parameters
                .strategic_work_order_parameters
                .insert(work_order_number, optimized_work_order);
        }
    }
}
            
