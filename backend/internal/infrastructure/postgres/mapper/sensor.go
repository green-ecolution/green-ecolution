package mapper

import (
	"encoding/json"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	mqtt "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/sensor/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalSensorRepoMapper interface {
	FromSql(src *sqlc.Sensor) (*sensor.Sensor, error)
	FromSqlList(src []*sqlc.Sensor) ([]*sensor.Sensor, error)
	FromSqlSensorData(src *sqlc.SensorDatum) (*sensor.SensorData, error)
	FromSqlSensorDataList(src []*sqlc.SensorDatum) ([]*sensor.SensorData, error)
	FromDomainSensorData(src *sensor.MqttPayload) *mqtt.MqttPayload
}

type InternalSensorRepoMapperImpl struct{}

func (c *InternalSensorRepoMapperImpl) FromSql(source *sqlc.Sensor) (*sensor.Sensor, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	return &sensor.Sensor{
		ID:             sensor.MustNewSensorID(source.ID),
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		Status:         MapSensorStatus(source.Status),
		Coordinate:     shared.MustNewCoordinate(source.Latitude, source.Longitude),
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}, nil
}

func (c *InternalSensorRepoMapperImpl) FromSqlList(source []*sqlc.Sensor) ([]*sensor.Sensor, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalSensorRepoMapperImpl) FromSqlSensorData(source *sqlc.SensorDatum) (*sensor.SensorData, error) {
	if source == nil {
		return nil, nil
	}
	data, err := MapSensorData(source.Data)
	if err != nil {
		return nil, err
	}
	return &sensor.SensorData{
		ID:        source.ID,
		SensorID:  sensor.MustNewSensorID(source.SensorID),
		CreatedAt: source.CreatedAt,
		UpdatedAt: source.UpdatedAt,
		Data:      data,
	}, nil
}

func (c *InternalSensorRepoMapperImpl) FromSqlSensorDataList(source []*sqlc.SensorDatum) ([]*sensor.SensorData, error) {
	return utils.MapSliceErr(source, c.FromSqlSensorData)
}

func (c *InternalSensorRepoMapperImpl) FromDomainSensorData(source *sensor.MqttPayload) *mqtt.MqttPayload {
	if source == nil {
		return nil
	}
	result := &mqtt.MqttPayload{
		Device:      source.Device,
		Battery:     source.Battery,
		Humidity:    source.Humidity,
		Temperature: source.Temperature,
	}
	if source.Watermarks != nil {
		result.Watermarks = make([]mqtt.Watermark, len(source.Watermarks))
		for i, w := range source.Watermarks {
			result.Watermarks[i] = mqtt.Watermark{
				Resistance: w.Resistance,
				Centibar:   w.Centibar,
				Depth:      w.Depth,
			}
		}
	}
	return result
}

func MapSensorData(src []byte) (*sensor.MqttPayload, error) {
	var payload sensor.MqttPayload
	err := json.Unmarshal(src, &payload)
	if err != nil {
		return nil, err
	}
	return &payload, nil
}

func MapSensorStatus(src sqlc.SensorStatus) sensor.SensorStatus {
	return sensor.SensorStatus(src)
}
