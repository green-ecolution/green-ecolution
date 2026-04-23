package sensor_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

var (
	currentTime     = time.Now()
	TestSensorID    = shared.MustNewSensorID("sensor-1")
	TestMqttPayload = &shared.MqttPayload{
		Device:      "sensor-123",
		Battery:     34.0,
		Humidity:    50,
		Temperature: 20,
		Watermarks: []shared.Watermark{
			{
				Resistance: 23,
				Centibar:   38,
				Depth:      30,
			},
			{
				Resistance: 23,
				Centibar:   38,
				Depth:      60,
			},
			{
				Resistance: 23,
				Centibar:   38,
				Depth:      90,
			},
		},
	}

	TestSensorData = &shared.SensorData{
		ID:        1,
		CreatedAt: currentTime,
		UpdatedAt: currentTime,
		Data:      TestMqttPayload,
	}

	TestSensor = &shared.Sensor{
		ID:         TestSensorID,
		CreatedAt:  currentTime,
		UpdatedAt:  currentTime,
		Coordinate: shared.MustNewCoordinate(54.82124518093376, 9.485702120628517),
		Status:     shared.SensorStatusOnline,
		LatestData: TestSensorData,
	}

	TestSensorList = []*shared.Sensor{
		TestSensor,
		{
			ID:         shared.MustNewSensorID("sensor-2"),
			CreatedAt:  currentTime,
			UpdatedAt:  currentTime,
			Coordinate: shared.MustNewCoordinate(54.78780993841013, 9.444052105200551),
			Status:     shared.SensorStatusOffline,
			LatestData: &shared.SensorData{},
		},
		{
			ID:         shared.MustNewSensorID("sensor-3"),
			CreatedAt:  currentTime,
			UpdatedAt:  currentTime,
			Coordinate: shared.MustNewCoordinate(54.77933725347423, 9.426465409018832),
			Status:     shared.SensorStatusUnknown,
			LatestData: &shared.SensorData{},
		},
		{
			ID:         shared.MustNewSensorID("sensor-4"),
			CreatedAt:  currentTime,
			UpdatedAt:  currentTime,
			Coordinate: shared.MustNewCoordinate(54.82078826498143, 9.489684366114483),
			Status:     shared.SensorStatusOnline,
			LatestData: &shared.SensorData{},
		},
	}
)
