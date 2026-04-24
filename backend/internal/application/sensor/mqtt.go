package sensor

import (
	"context"
	"errors"

	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

func (s *SensorService) HandleMessage(ctx context.Context, payload *sensorDomain.MqttPayload) (*sensorDomain.SensorData, error) {
	log := logger.GetLogger(ctx)
	if payload == nil {
		log.Debug("mqtt payload is nil")
		return nil, errors.New("mqtt payload is nil")
	}

	if err := s.validator.Struct(payload); err != nil {
		log.Debug("failed to validate mqtt payload struct", "error", err)
		return nil, err
	}

	sensorID, err := sensorDomain.NewSensorID(payload.Device)
	if err != nil {
		log.Error("failed to create sensor id from mqtt payload", "device", payload.Device, "error", err)
		return nil, err
	}

	coord, err := shared.NewCoordinate(payload.Latitude, payload.Longitude)
	if err != nil {
		log.Error("failed to create coordinate from mqtt payload", "latitude", payload.Latitude, "longitude", payload.Longitude, "error", err)
		return nil, err
	}

	sen, err := s.sensorRepo.GetByID(ctx, sensorID)
	if err != nil {
		var entityNotFoundErr shared.ErrEntityNotFound
		if !errors.As(err, &entityNotFoundErr) {
			log.Error("failed to get sensor by id", "error", err)
			return nil, err
		}
	}

	if sen != nil {
		updatedSensor, err := s.updateSensorCoordsAndStatus(ctx, coord, sen)
		if err != nil {
			log.Error("failed to update sensor", "error", err)
			return nil, err
		}
		sen = updatedSensor
	} else {
		log.Info("a new sensor has joined the party! creating sensor record", "sensor_id", sensorID.String(), "sensor_latitude", coord.Latitude(), "sensor_longitude", coord.Longitude())
		createdSensor, err := s.sensorRepo.Create(ctx, &sensorDomain.Sensor{
			ID:         sensorID,
			Coordinate: coord,
			Status:     sensorDomain.SensorStatusOnline,
		})
		if err != nil {
			log.Error("failed to create sensor", "error", err)
			return nil, err
		}
		sen = createdSensor
	}

	data := sensorDomain.SensorData{
		Data: payload,
	}
	err = s.sensorRepo.InsertSensorData(ctx, &data, sen.ID)
	if err != nil {
		log.Error("failed to insert sensor data", "sensor_id", sen.ID.String(), "error", err)
		return nil, err
	}

	sensorData, err := s.sensorRepo.GetLatestSensorDataBySensorID(ctx, sen.ID)
	if err != nil {
		return nil, err
	}

	err = s.MapSensorToTree(ctx, sen)
	if err != nil {
		return nil, err
	}

	s.publishNewSensorDataEvent(ctx, sensorData)
	return sensorData, nil
}

func (s *SensorService) updateSensorCoordsAndStatus(ctx context.Context, coord shared.Coordinate, sen *sensorDomain.Sensor) (*sensorDomain.Sensor, error) {
	log := logger.GetLogger(ctx)
	if sen.Coordinate.Latitude() != coord.Latitude() || sen.Coordinate.Longitude() != coord.Longitude() || sen.Status != sensorDomain.SensorStatusOnline {
		sen.Coordinate = coord
		sen.Status = sensorDomain.SensorStatusOnline
		updatedSensor, err := s.sensorRepo.Update(ctx, sen.ID, sen)
		if err != nil {
			return nil, err
		}
		log.Info("coordinates and status of sensor have been updated successfully", "sensor_id", updatedSensor.ID.String())
		return updatedSensor, err
	}

	log.Debug("sensor don't need to update coordinates and status")
	return sen, nil
}
