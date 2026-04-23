package entities

type Evaluation struct {
	TreeCount             int64
	TreeClusterCount      int64
	SensorCount           int64
	WateringPlanCount     int64
	TotalWaterConsumption int64
	UserWateringPlanCount int64
	VehicleEvaluation     []*VehicleEvaluation
	RegionEvaluation      []*RegionEvaluation
}

type VehicleEvaluation struct {
	NumberPlate       string
	WateringPlanCount int64
}

type RegionEvaluation struct {
	Name              string
	WateringPlanCount int64
}
