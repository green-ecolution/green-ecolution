package sensor

import (
	"context"
	"errors"

	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

func (s *SensorService) HandleMessage(ctx context.Context, payload *entities.MqttPayload) (*entities.SensorData, error) {
	log := logger.GetLogger(ctx)
	if payload == nil {
		log.Debug("mqtt payload is nil")
		return nil, errors.New("mqtt payload is nil")
	}

	if err := s.validator.Struct(payload); err != nil {
		log.Debug("failed to validate mqtt payload struct", "error", err)
		return nil, err
	}

	sensorID, err := entities.NewSensorID(payload.Device)
	if err != nil {
		log.Error("failed to create sensor id from mqtt payload", "device", payload.Device, "error", err)
		return nil, err
	}

	coord, err := entities.NewCoordinate(payload.Latitude, payload.Longitude)
	if err != nil {
		log.Error("failed to create coordinate from mqtt payload", "latitude", payload.Latitude, "longitude", payload.Longitude, "error", err)
		return nil, err
	}

	sensor, err := s.sensorRepo.GetByID(ctx, sensorID)
	if err != nil {
		var entityNotFoundErr entities.ErrEntityNotFound
		if !errors.As(err, &entityNotFoundErr) {
			log.Error("failed to get sensor by id", "error", err)
			return nil, err
		}
	}

	if sensor != nil {
		updatedSensor, err := s.updateSensorCoordsAndStatus(ctx, coord, sensor)
		if err != nil {
			log.Error("failed to update sensor", "error", err)
			return nil, err
		}
		sensor = updatedSensor
	} else {
		log.Info("a new sensor has joined the party! creating sensor record", "sensor_id", sensorID.String(), "sensor_latitude", coord.Latitude(), "sensor_longitude", coord.Longitude())
		createdSensor, err := s.sensorRepo.Create(ctx, func(s *entities.Sensor, _ entities.SensorRepository) (bool, error) {
			s.ID = sensorID
			s.Coordinate = coord
			s.Status = entities.SensorStatusOnline
			return true, nil
		})
		if err != nil {
			log.Error("failed to create sensor", "error", err)
			return nil, err
		}
		sensor = createdSensor
	}

	data := entities.SensorData{
		Data: payload,
	}
	err = s.sensorRepo.InsertSensorData(ctx, &data, sensor.ID)
	if err != nil {
		log.Error("failed to insert sensor data", "sensor_id", sensor.ID.String(), "error", err)
		return nil, err
	}

	sensorData, err := s.sensorRepo.GetLatestSensorDataBySensorID(ctx, sensor.ID)
	if err != nil {
		return nil, err
	}

	err = s.MapSensorToTree(ctx, sensor)
	if err != nil {
		return nil, err
	}

	s.publishNewSensorDataEvent(ctx, sensorData)
	return sensorData, nil
}

func (s *SensorService) updateSensorCoordsAndStatus(ctx context.Context, coord entities.Coordinate, sensor *entities.Sensor) (*entities.Sensor, error) {
	log := logger.GetLogger(ctx)
	if sensor.Coordinate.Latitude() != coord.Latitude() || sensor.Coordinate.Longitude() != coord.Longitude() || sensor.Status != entities.SensorStatusOnline {
		updatedSensor, err := s.sensorRepo.Update(ctx, sensor.ID, func(s *entities.Sensor, _ entities.SensorRepository) (bool, error) {
			s.Coordinate = coord
			s.Status = entities.SensorStatusOnline
			return true, nil
		})
		if err != nil {
			return nil, err
		}
		log.Info("coordinates and status of sensor have been updated successfully", "sensor_id", updatedSensor.ID.String())
		return updatedSensor, err
	}

	log.Debug("sensor don't need to update coordinates and status")
	return sensor, nil
}
