package sensor_test

import (
	"time"

	"github.com/green-ecolution/backend/internal/entities"
)

var (
	currentTime     = time.Now()
	TestSensorID    = "sensor-1"
	TestMqttPayload = &entities.MqttPayload{
		Device:      "sensor-123",
		Battery:     34.0,
		Humidity:    50,
		Temperature: 20,
		Watermarks: []entities.Watermark{
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

	TestSensorData = &entities.SensorData{
		ID:        1,
		CreatedAt: currentTime,
		UpdatedAt: currentTime,
		Data:      TestMqttPayload,
	}

	TestSensor = &entities.Sensor{
		ID:         TestSensorID,
		CreatedAt:  currentTime,
		UpdatedAt:  currentTime,
		Latitude:   54.82124518093376,
		Longitude:  9.485702120628517,
		Status:     entities.SensorStatusOnline,
		LatestData: TestSensorData,
	}

	TestSensorList = []*entities.Sensor{
		TestSensor,
		{
			ID:         "sensor-2",
			CreatedAt:  currentTime,
			UpdatedAt:  currentTime,
			Latitude:   54.78780993841013,
			Longitude:  9.444052105200551,
			Status:     entities.SensorStatusOffline,
			LatestData: &entities.SensorData{},
		},
		{
			ID:         "sensor-3",
			CreatedAt:  currentTime,
			UpdatedAt:  currentTime,
			Latitude:   54.77933725347423,
			Longitude:  9.426465409018832,
			Status:     entities.SensorStatusUnknown,
			LatestData: &entities.SensorData{},
		},
		{
			ID:         "sensor-4",
			CreatedAt:  currentTime,
			UpdatedAt:  currentTime,
			Latitude:   54.82078826498143,
			Longitude:  9.489684366114483,
			Status:     entities.SensorStatusOnline,
			LatestData: &entities.SensorData{},
		},
	}
)
