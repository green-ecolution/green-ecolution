package mapper

import (
	"encoding/json"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	mqtt "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/sensor/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalSensorRepoMapper interface {
	FromSql(src *sqlc.Sensor) (*shared.Sensor, error)
	FromSqlList(src []*sqlc.Sensor) ([]*shared.Sensor, error)
	FromSqlSensorData(src *sqlc.SensorDatum) (*shared.SensorData, error)
	FromSqlSensorDataList(src []*sqlc.SensorDatum) ([]*shared.SensorData, error)
	FromDomainSensorData(src *shared.MqttPayload) *mqtt.MqttPayload
}

type InternalSensorRepoMapperImpl struct{}

func (c *InternalSensorRepoMapperImpl) FromSql(source *sqlc.Sensor) (*shared.Sensor, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	return &shared.Sensor{
		ID:             shared.MustNewSensorID(source.ID),
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		Status:         MapSensorStatus(source.Status),
		Coordinate:     shared.MustNewCoordinate(source.Latitude, source.Longitude),
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}, nil
}

func (c *InternalSensorRepoMapperImpl) FromSqlList(source []*sqlc.Sensor) ([]*shared.Sensor, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalSensorRepoMapperImpl) FromSqlSensorData(source *sqlc.SensorDatum) (*shared.SensorData, error) {
	if source == nil {
		return nil, nil
	}
	data, err := MapSensorData(source.Data)
	if err != nil {
		return nil, err
	}
	return &shared.SensorData{
		ID:        source.ID,
		SensorID:  shared.MustNewSensorID(source.SensorID),
		CreatedAt: source.CreatedAt,
		UpdatedAt: source.UpdatedAt,
		Data:      data,
	}, nil
}

func (c *InternalSensorRepoMapperImpl) FromSqlSensorDataList(source []*sqlc.SensorDatum) ([]*shared.SensorData, error) {
	return utils.MapSliceErr(source, c.FromSqlSensorData)
}

func (c *InternalSensorRepoMapperImpl) FromDomainSensorData(source *shared.MqttPayload) *mqtt.MqttPayload {
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

func MapSensorData(src []byte) (*shared.MqttPayload, error) {
	var payload shared.MqttPayload
	err := json.Unmarshal(src, &payload)
	if err != nil {
		return nil, err
	}
	return &payload, nil
}

func MapSensorStatus(src sqlc.SensorStatus) shared.SensorStatus {
	return shared.SensorStatus(src)
}
