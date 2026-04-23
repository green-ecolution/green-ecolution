package entities

type Entities interface {
	Sensor |
		Vehicle |
		TreeCluster |
		Tree |
		Region |
		WateringPlan |
		Evaluation
}

type EntityFunc[T Entities] func(*T)
