package mapper

import (
	"encoding/json"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	mqtt "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/sensor/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalSensorRepoMapper interface {
	FromSql(src *sqlc.Sensor) (*entities.Sensor, error)
	FromSqlList(src []*sqlc.Sensor) ([]*entities.Sensor, error)
	FromSqlSensorData(src *sqlc.SensorDatum) (*entities.SensorData, error)
	FromSqlSensorDataList(src []*sqlc.SensorDatum) ([]*entities.SensorData, error)
	FromDomainSensorData(src *entities.MqttPayload) *mqtt.MqttPayload
}

type InternalSensorRepoMapperImpl struct{}

func (c *InternalSensorRepoMapperImpl) FromSql(source *sqlc.Sensor) (*entities.Sensor, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	return &entities.Sensor{
		ID:             entities.MustNewSensorID(source.ID),
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		Status:         MapSensorStatus(source.Status),
		Coordinate:     entities.MustNewCoordinate(source.Latitude, source.Longitude),
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}, nil
}

func (c *InternalSensorRepoMapperImpl) FromSqlList(source []*sqlc.Sensor) ([]*entities.Sensor, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func (c *InternalSensorRepoMapperImpl) FromSqlSensorData(source *sqlc.SensorDatum) (*entities.SensorData, error) {
	if source == nil {
		return nil, nil
	}
	data, err := MapSensorData(source.Data)
	if err != nil {
		return nil, err
	}
	return &entities.SensorData{
		ID:        source.ID,
		SensorID:  entities.MustNewSensorID(source.SensorID),
		CreatedAt: source.CreatedAt,
		UpdatedAt: source.UpdatedAt,
		Data:      data,
	}, nil
}

func (c *InternalSensorRepoMapperImpl) FromSqlSensorDataList(source []*sqlc.SensorDatum) ([]*entities.SensorData, error) {
	return utils.MapSliceErr(source, c.FromSqlSensorData)
}

func (c *InternalSensorRepoMapperImpl) FromDomainSensorData(source *entities.MqttPayload) *mqtt.MqttPayload {
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

func MapSensorData(src []byte) (*entities.MqttPayload, error) {
	var payload entities.MqttPayload
	err := json.Unmarshal(src, &payload)
	if err != nil {
		return nil, err
	}
	return &payload, nil
}

func MapSensorStatus(src sqlc.SensorStatus) entities.SensorStatus {
	return entities.SensorStatus(src)
}
