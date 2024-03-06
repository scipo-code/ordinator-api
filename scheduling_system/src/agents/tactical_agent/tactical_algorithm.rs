pub struct TacticalAlgorithm {
    objective_value: f32,
}

impl TacticalAlgorithm {
    pub fn new() -> Self {
        TacticalAlgorithm {
            objective_value: 0.0,
        }
    }
}

impl TacticalAlgorithm {
    pub fn status(&self) -> String {
        "TacticalAlgorithm is running".to_string()
    }

    pub fn get_objective_value(&self) -> f32 {
        self.objective_value
    }
}
