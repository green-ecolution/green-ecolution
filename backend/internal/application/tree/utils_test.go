package tree_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	testLatitude     = 9.446741
	testLongitude    = 54.801539
	testCoordinate   = shared.MustNewCoordinate(testLatitude, testLongitude)
	TestTreeClusters = []*shared.TreeCluster{
		{
			ID:            1,
			CreatedAt:     time.Now(),
			UpdatedAt:     time.Now(),
			Name:          "Cluster 1",
			Address:       "123 Main St",
			Description:   "Test description",
			SoilCondition: shared.TreeSoilConditionLehmig,
			Archived:      false,
			Coordinate:    &testCoordinate,
			Trees:         TestTreesList,
		},
		{
			ID:            2,
			CreatedAt:     time.Now(),
			UpdatedAt:     time.Now(),
			Name:          "Cluster 2",
			Address:       "456 Second St",
			Description:   "Test description",
			SoilCondition: shared.TreeSoilConditionSandig,
			Archived:      false,
			Coordinate:    nil,
			Trees:         []*shared.Tree{},
			LastWatered:   nil,
		},
	}

	TestTreesList = []*shared.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			PlantingYear:   shared.MustNewPlantingYear(2023),
			WateringStatus: shared.WateringStatusBad,
			LastWatered:    nil,
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     shared.MustNewCoordinate(9.446700, 54.801510),
			Description:    "A young pine tree",
			PlantingYear:   shared.MustNewPlantingYear(2023),
			WateringStatus: shared.WateringStatusUnknown,
			LastWatered:    nil,
		},
	}

	testFilterTrees = []*shared.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			TreeCluster:    TestTreeClusters[0],
			WateringStatus: shared.WateringStatusGood,
			PlantingYear:   shared.MustNewPlantingYear(2023),
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			TreeCluster:    TestTreeClusters[0],
			Description:    "A young pine tree",
			WateringStatus: shared.WateringStatusBad,
			PlantingYear:   shared.MustNewPlantingYear(2022),
		},
	}

	TestSensors = []*shared.Sensor{
		{
			ID:         shared.MustNewSensorID("sensor-1"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Status:     shared.SensorStatusUnknown,
			Coordinate: shared.MustNewCoordinate(54.82124518093376, 9.485702120628517),
			LatestData: TestSensorDataBad,
		},
		{
			ID:         shared.MustNewSensorID("sensor-2"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Status:     shared.SensorStatusUnknown,
			Coordinate: shared.MustNewCoordinate(54.787809938410133, 9.444052105200551),
			LatestData: &shared.SensorData{},
		},
	}

	TestTreeCreate = &shared.TreeCreate{
		Species:       "Oak",
		Coordinate:    shared.MustNewCoordinate(testLatitude, testLongitude),
		PlantingYear:  shared.MustNewPlantingYear(2023),
		Number:        "T001",
		Description:   "Test tree description",
		TreeClusterID: utils.P(int32(1)),
		SensorID:      utils.P(shared.MustNewSensorID("sensor-1")),
	}

	TestTreeUpdate = &shared.TreeUpdate{
		TreeClusterID: utils.P(int32(1)),
		SensorID:      utils.P(shared.MustNewSensorID("sensor-1")),
		PlantingYear:  shared.MustNewPlantingYear(2023),
		Species:       "Oak",
		Number:        "T001",
		Coordinate:    shared.MustNewCoordinate(testLatitude, testLongitude),
		Description:   "Updated description",
	}

	TestSensorDataBad = &shared.SensorData{
		ID:        1,
		SensorID:  shared.MustNewSensorID("sensor-1"),
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Data: &shared.MqttPayload{
			Device:      "sensor-1",
			Temperature: 2.0,
			Humidity:    0.5,
			Battery:     3.3,
			Watermarks: []shared.Watermark{
				{
					Resistance: 2000,
					Centibar:   80,
					Depth:      30,
				},
				{
					Resistance: 2200,
					Centibar:   85,
					Depth:      60,
				},
				{
					Resistance: 2500,
					Centibar:   90,
					Depth:      90,
				},
			},
		},
	}
)
