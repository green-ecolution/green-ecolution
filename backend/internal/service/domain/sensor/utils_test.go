package sensor_test

import (
	"time"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
)

var (
	TestListMQTTPayload = []*domain.MqttPayload{
		{
			Device:      "sensor001",
			Battery:     45.3,
			Humidity:    0.75,
			Temperature: 22.5,
			Latitude:    37.7749,
			Longitude:   -122.4194,
			Watermarks: []domain.Watermark{
				{Centibar: 30, Resistance: 20, Depth: 30},
				{Centibar: 40, Resistance: 25, Depth: 60},
				{Centibar: 50, Resistance: 30, Depth: 90},
			},
		},
		{
			Device:      "sensor002",
			Battery:     78.9,
			Humidity:    0.60,
			Temperature: 18.3,
			Latitude:    48.8566,
			Longitude:   2.3522,
			Watermarks: []domain.Watermark{
				{Centibar: 25, Resistance: 18, Depth: 30},
				{Centibar: 35, Resistance: 22, Depth: 60},
				{Centibar: 45, Resistance: 27, Depth: 90},
			},
		},
		{
			Device:      "sensor003",
			Battery:     32.1,
			Humidity:    0.85,
			Temperature: 28.0,
			Latitude:    -33.8688,
			Longitude:   151.2093,
			Watermarks: []domain.Watermark{
				{Centibar: 20, Resistance: 15, Depth: 30},
				{Centibar: 30, Resistance: 20, Depth: 60},
				{Centibar: 40, Resistance: 25, Depth: 90},
			},
		},
	}

	TestMQTTPayLoadInvalidLong = &domain.MqttPayload{
		Device:      "sensor001",
		Battery:     45.3,
		Humidity:    0.75,
		Temperature: 22.5,
		Latitude:    37.7749,
		Longitude:   181.0, // invalid
		Watermarks: []domain.Watermark{
			{Centibar: 30, Resistance: 20, Depth: 30},
			{Centibar: 40, Resistance: 25, Depth: 60},
			{Centibar: 50, Resistance: 30, Depth: 90},
		},
	}

	TestMQTTPayLoadInvalidLat = &domain.MqttPayload{
		Device:      "sensor001",
		Battery:     45.3,
		Humidity:    0.75,
		Temperature: 22.5,
		Latitude:    91.0, // invalid
		Longitude:   -122.4194,
		Watermarks: []domain.Watermark{
			{Centibar: 30, Resistance: 20, Depth: 30},
			{Centibar: 40, Resistance: 25, Depth: 60},
			{Centibar: 50, Resistance: 30, Depth: 90},
		},
	}

	TestSensor = &domain.Sensor{
		ID:         domain.MustNewSensorID("sensor001"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: domain.MustNewCoordinate(54.82124518093376, 9.485702120628517),
		Status:     domain.SensorStatusOnline,
		LatestData: TestSensorData[0],
	}

	TestSensorData = []*domain.SensorData{
		{
			ID:        1,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestListMQTTPayload[0],
		},
		{
			ID:        2,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestListMQTTPayload[1],
		},
		{
			ID:        3,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
			Data:      TestListMQTTPayload[2],
		},
	}

	TestSensorList = []*domain.Sensor{
		TestSensor,
		{
			ID:         domain.MustNewSensorID("sensor-2"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Coordinate: domain.MustNewCoordinate(54.78780993841013, 9.444052105200551),
			Status:     domain.SensorStatusOffline,
			LatestData: &domain.SensorData{},
		},
		{
			ID:         domain.MustNewSensorID("sensor-3"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Coordinate: domain.MustNewCoordinate(54.77933725347423, 9.426465409018832),
			Status:     domain.SensorStatusUnknown,
			LatestData: &domain.SensorData{},
		},
		{
			ID:         domain.MustNewSensorID("sensor-4"),
			CreatedAt:  time.Now(),
			UpdatedAt:  time.Now(),
			Coordinate: domain.MustNewCoordinate(54.82078826498143, 9.489684366114483),
			Status:     domain.SensorStatusOnline,
			LatestData: &domain.SensorData{},
		},
	}

	TestSensorNearestTree = &domain.Sensor{
		ID:         domain.MustNewSensorID("sensor-05"),
		CreatedAt:  time.Now(),
		UpdatedAt:  time.Now(),
		Coordinate: domain.MustNewCoordinate(54.821535, 9.487200),
		Status:     domain.SensorStatusOnline,
		LatestData: TestSensorData[0],
	}

	TestNearestTree = &domain.Tree{
		ID:           5,
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
		Species:      "Oak",
		Number:       "T001",
		Coordinate:   domain.MustNewCoordinate(54.8215076622281, 9.487153277881877),
		Description:  "A mature oak tree",
		PlantingYear: domain.MustNewPlantingYear(2023),
	}
)
