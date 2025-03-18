#[derive(Eq, PartialEq, Default, Serialize, Deserialize, Debug, Clone)]
pub struct TacticalResources {
    #[serde(with = "any_key_map")]
    pub resources: HashMap<Resources, Days>,
}
impl TacticalResources {
    pub fn new(resources: HashMap<Resources, Days>) -> Self {
        TacticalResources { resources }
    }

    pub fn get_resource(&self, resource: &Resources, day: &Day) -> &Work {
        self.resources.get(resource).unwrap().get(day)
    }

    pub fn new_from_data(resources: Vec<Resources>, tactical_days: Vec<Day>, load: Work) -> Self {
        let mut resource_capacity: HashMap<Resources, Days> = HashMap::new();
        for resource in resources {
            let mut days = HashMap::new();
            for day in tactical_days.iter() {
                days.insert(day.clone(), load);
            }

            resource_capacity.insert(resource, Days { days });
        }
        TacticalResources::new(resource_capacity)
    }

    pub fn update_resources(&mut self, resources: Self) {
        for resource in resources.resources {
            for day in resource.1.days {
                *self
                    .resources
                    .get_mut(&resource.0)
                    .unwrap()
                    .days
                    .get_mut(&day.0)
                    .unwrap() = day.1;
            }
        }
    }

    pub fn determine_period_load(
        &self,
        resource: &Resources,
        period: &ordinator_scheduling_environment::time_environment::period::Period,
    ) -> Result<Work> {
        let days = &self
            .resources
            .get(resource)
            .with_context(|| "The resources between the strategic and the tactical should always correspond, unless that the tactical has not been initialized yet".to_string())?
            .days;

        Ok(days
            .iter()
            .filter(|(day, _)| period.contains_date(day.date().date_naive()))
            .map(|(_, work)| work)
            .fold(Work::from(0.0), |acc, work| &acc + work))
    }
}

impl From<&MutexGuard<SchedulingEnvironment>> for TacticalResources {
    fn from(value: &MutexGuard<SchedulingEnvironment>) -> Self {
        todo!()
    }
}

pub fn generate_tactical_resources(
    &self,
    days: &[Day],
    _empty_full: EmptyFull,
) -> TacticalResources {
    // TODO [ ]
    // Move this out of the code and into `configuration`
    let _hours_per_day = 6.0;

    let gradual_reduction = |i: usize| -> f64 {
        match i {
            0..=13 => 1.0,
            14..=27 => 1.0,
            _ => 1.0,
        }
    };

    // WARN
    // Should this be multi skill?
    let mut tactical_resources_inner = HashMap::<Resources, Days>::new();
    for operational_configuration_all in self.agent_environment.operational.values() {
        for (i, day) in days.iter().enumerate() {
            let resource_periods = tactical_resources_inner
                // FIX
                // WARN
                // There is a logic error here. If we want to compare with the `StrategicAgent`.
                .entry(operational_configuration_all.id.1.first().cloned().unwrap())
                .or_insert(Days::new(HashMap::new()));

            *resource_periods
                .days
                .entry(day.clone())
                .or_insert_with(|| Work::from(0.0)) +=
                Work::from(operational_configuration_all.hours_per_day * gradual_reduction(i));
        }
    }
    TacticalResources::new(tactical_resources_inner)
}
