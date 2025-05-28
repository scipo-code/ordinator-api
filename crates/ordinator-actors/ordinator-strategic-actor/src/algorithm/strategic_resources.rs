use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::OperationalId;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

// Where should the operational struct be found? I think that it should
// be in the shared types. You should not deserialize this. You cannot
// code software with this mentality.
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StrategicResources(pub HashMap<Period, HashMap<OperationalId, OperationalResource>>);

impl<'a> From<(&MutexGuard<'a, SchedulingEnvironment>, &Id)> for StrategicResources
{
    fn from(value: (&MutexGuard<'a, SchedulingEnvironment>, &Id)) -> Self
    {
        let gradual_reduction = |i: usize| -> f64 {
            if i == 0 {
                1.0
            } else if i == 1 {
                0.9
            } else if i == 2 {
                0.8
            } else {
                0.6
            }
        };

        // You cannot create this without the ID of the Actor, as you do not know
        // who to write here.
        let mut strategic_resources_inner =
            HashMap::<Period, HashMap<OperationalId, OperationalResource>>::new();

        for (i, period) in value.0.time_environment.periods.iter().enumerate() {
            let mut operational_resource_map = HashMap::new();
            for operational_agent in &value
                .0
                .worker_environment
                .actor_specification
                .get(value.1.asset())
                .with_context(|| {
                    format!("Missing Actor: {:?} in the SchedulingEnvironment", value.1)
                })
                .expect("Missing the required Actor")
                .operational
            {
                // What is it that you are trying to do here? You want to instantiate an agent
                // TODO: Could you reuse the OperationalResource. No could you inplement a
                // into formulation here? I think that is a that ... THis is actually fun!
                let mut skill_hours: HashMap<Resources, Work> = HashMap::new();

                // let availability = &operational_agent.operational_configuration.availability;

                // This does not make any sense for the longer term. I think that you should
                // rely on the 13 days.
                let days_in_period = 13.0; // WARN: period.count_overlapping_days(availability);

                for resource in &operational_agent.id.1 {
                    skill_hours.insert(
                        *resource,
                        Work::from(
                            operational_agent.hours_per_day * days_in_period * gradual_reduction(i),
                        ),
                    );
                }

                let operational_resource = OperationalResource::new(
                    &operational_agent.id.0,
                    Work::from(
                        operational_agent.hours_per_day * days_in_period * gradual_reduction(i),
                    ),
                    operational_agent.id.1.clone(),
                );

                operational_resource_map
                    .insert(operational_agent.id.0.clone(), operational_resource);
            }
            strategic_resources_inner.insert(period.clone(), operational_resource_map);
        }

        StrategicResources::new(strategic_resources_inner)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct OperationalResource
{
    pub id: OperationalId,
    pub total_hours: Work,
    pub skill_hours: HashMap<Resources, Work>,
}

impl OperationalResource
{
    pub fn new(id: &str, total_hours: Work, skills: Vec<Resources>) -> Self
    {
        let skill_hours: HashMap<Resources, Work> =
            skills.iter().map(|ski| (*ski, total_hours)).collect();

        Self {
            id: id.to_string(),
            total_hours,
            skill_hours,
        }
    }
}

impl StrategicResources
{
    pub fn assert_well_shaped_resources(&self) -> Result<()>
    {
        for period in &self.0 {
            for operational_resource in period.1 {
                let total_hours = operational_resource.1.total_hours;
                ensure!(
                    operational_resource
                        .1
                        .skill_hours
                        .values()
                        .all(|wor| *wor == total_hours),
                    format!(
                        "StrategicResources are not well shaped: {:#?}",
                        operational_resource.1
                    )
                )
            }
        }
        Ok(())
    }

    pub fn insert_operational_resource(
        &mut self,
        period: Period,
        operational_resource: OperationalResource,
    )
    {
        let operational_key = operational_resource.id.clone();
        self.0
            .entry(period)
            .and_modify(|ele| {
                ele.insert(operational_key.clone(), operational_resource.clone());
            })
            .or_insert_with(|| HashMap::from([(operational_key, operational_resource)]));
    }
}

impl StrategicResources
{
    pub fn new(resources: HashMap<Period, HashMap<OperationalId, OperationalResource>>) -> Self
    {
        Self(resources)
    }

    // Okay so you have to determine a good way of updating the load here. The best
    // approach would probably be to create a small heuristic
    //
    // The load should be updated and this means that we need to generate a small
    // heuristic. As this is no longer deterministic.
    pub fn update_load(
        &mut self,
        period: &Period,
        resource: Resources,
        load: Work,
        operational_resource: &OperationalResource,
        load_operation: LoadOperation,
    )
    {
        let period_entry = self.0.entry(period.clone());
        let operational = match period_entry {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(HashMap::new()),
        };

        match operational.entry(operational_resource.id.clone()) {
            Entry::Occupied(mut operational_resource) => match load_operation {
                LoadOperation::Add => {
                    let previous_total_hours = operational_resource.get().total_hours;
                    operational_resource
                        .get_mut()
                        .skill_hours
                        .entry(resource)
                        .or_insert(previous_total_hours);

                    operational_resource.get_mut().total_hours += load;

                    operational_resource
                        .get_mut()
                        .skill_hours
                        .iter_mut()
                        .for_each(|ski_loa| *ski_loa.1 += load);
                }
                LoadOperation::Sub => {
                    let previous_total_hours = operational_resource.get().total_hours;
                    operational_resource
                        .get_mut()
                        .skill_hours
                        .entry(resource)
                        .or_insert(previous_total_hours);
                    operational_resource.get_mut().total_hours -= load;
                    operational_resource
                        .get_mut()
                        .skill_hours
                        .iter_mut()
                        .for_each(|ski_loa| *ski_loa.1 -= load);
                }
            },
            Entry::Vacant(operational_resource_entry) => match load_operation {
                LoadOperation::Add => {
                    let total_load_hours = Work::from(load.to_f64());

                    let operational_resource = OperationalResource::new(
                        &operational_resource.id,
                        total_load_hours,
                        operational_resource
                            .skill_hours
                            .keys()
                            .chain(std::iter::once(&resource))
                            .cloned()
                            .collect(),
                    );

                    operational_resource_entry.insert(operational_resource);
                }
                LoadOperation::Sub => {
                    let total_load_hours = Work::from(-load.to_f64());

                    let operational_resource = OperationalResource::new(
                        &operational_resource.id,
                        total_load_hours,
                        operational_resource
                            .skill_hours
                            .keys()
                            .chain(std::iter::once(&resource))
                            .cloned()
                            .collect(),
                    );
                    operational_resource_entry.insert(operational_resource);
                }
            },
        };
    }

    pub fn update_resource_capacities(&mut self, resources: Self) -> Result<()>
    {
        for period in &resources.0 {
            for operational in period.1 {
                self.0
                    .entry(period.0.clone())
                    .or_default()
                    .entry(operational.0.clone())
                    .and_modify(|existing| *existing = operational.1.clone())
                    .or_insert(operational.1.clone());
            }
        }
        Ok(())
    }

    pub fn initialize_resource_loadings(&mut self, resources: Self)
    {
        for period in resources.0 {
            for operational in period.1 {
                let mut operational_resource = operational.1;

                operational_resource.total_hours = Work::from(0.0);

                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|ele| *ele.1 = Work::from(0.0));

                self.0
                    .entry(period.0.clone())
                    .or_default()
                    .entry(operational.0.clone())
                    .and_modify(|existing| *existing = operational_resource.clone())
                    .or_insert(operational_resource);
            }
        }
    }

    pub fn aggregated_capacity_by_period_and_resource(
        &self,
        period: &Period,
        resource: &Resources,
    ) -> Result<Work>
    {
        Ok(self
            .0
            .get(period)
            .with_context(|| {
                format!(
                    "{} not found is {:?}",
                    period,
                    std::any::type_name::<StrategicResources>()
                )
            })?
            // WARN START HERE
            .values()
            .fold(Work::from(0.0), |acc, or| {
                acc + *or.skill_hours.get(resource).unwrap_or(&Work::from(0.0))
            }))
    }
}
