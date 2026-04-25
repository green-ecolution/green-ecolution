use crate::domain::RepositoryError;

#[derive(Debug, Clone)]
pub struct Evaluation {
    tree_count: u32,
    cluster_count: u32,
    sensor_count: u32,
    watering_plan_count: u32,
    user_watering_plan_count: u32,
    vehicle_evaluation: Vec<VehicleEvaluation>,
    region_evaluation: Vec<RegionEvaluation>,
}

impl Evaluation {
    pub fn new(
        tree_count: u32,
        cluster_count: u32,
        sensor_count: u32,
        watering_plan_count: u32,
        user_watering_plan_count: u32,
        vehicle_evaluation: Vec<VehicleEvaluation>,
        region_evaluation: Vec<RegionEvaluation>,
    ) -> Self {
        Self {
            tree_count,
            cluster_count,
            sensor_count,
            watering_plan_count,
            user_watering_plan_count,
            vehicle_evaluation,
            region_evaluation,
        }
    }

    pub fn tree_count(&self) -> u32 {
        self.tree_count
    }
    pub fn cluster_count(&self) -> u32 {
        self.cluster_count
    }
    pub fn sensor_count(&self) -> u32 {
        self.sensor_count
    }
    pub fn watering_plan_count(&self) -> u32 {
        self.watering_plan_count
    }
    pub fn user_watering_plan_count(&self) -> u32 {
        self.user_watering_plan_count
    }
    pub fn vehicle_evaluation(&self) -> &[VehicleEvaluation] {
        &self.vehicle_evaluation
    }
    pub fn region_evaluation(&self) -> &[RegionEvaluation] {
        &self.region_evaluation
    }
}

#[derive(Debug, Clone)]
pub struct VehicleEvaluation {
    number_plate: String,
    watering_plan_count: u32,
}

impl VehicleEvaluation {
    pub fn new(number_plate: String, watering_plan_count: u32) -> Self {
        Self {
            number_plate,
            watering_plan_count,
        }
    }

    pub fn number_plate(&self) -> &str {
        &self.number_plate
    }
    pub fn watering_plan_count(&self) -> u32 {
        self.watering_plan_count
    }
}

#[derive(Debug, Clone)]
pub struct RegionEvaluation {
    name: String,
    watering_plan_count: u32,
}

impl RegionEvaluation {
    pub fn new(name: String, watering_plan_count: u32) -> Self {
        Self {
            name,
            watering_plan_count,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn watering_plan_count(&self) -> u32 {
        self.watering_plan_count
    }
}

#[async_trait::async_trait]
pub trait EvaluationRepository: Send + Sync {
    async fn regions_with_watering_plan(&self) -> Result<Vec<RegionEvaluation>, RepositoryError>;
    async fn vehicle_with_watering_plan(&self) -> Result<Vec<VehicleEvaluation>, RepositoryError>;
    async fn total_consumed_water(&self) -> Result<f64, RepositoryError>;
    async fn watering_plan_user(&self) -> Result<u64, RepositoryError>;
}
