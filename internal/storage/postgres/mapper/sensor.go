package mapper

import (
	"encoding/json"

	"github.com/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	mqtt "github.com/green-ecolution/backend/internal/storage/postgres/sensor/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:PgTimestampToTime
// goverter:extend github.com/green-ecolution/backend/internal/utils:PgTimestampToTimePtr
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/backend/internal/utils:StringPtrToString
// goverter:extend MapSensorStatus MapSensorData
type InternalSensorRepoMapper interface {
	// goverter:ignore LatestData
	// goverter:map AdditionalInformations AdditionalInfo  | github.com/green-ecolution/backend/internal/utils:MapAdditionalInfo
	FromSql(src *sqlc.Sensor) (*entities.Sensor, error)
	FromSqlList(src []*sqlc.Sensor) ([]*entities.Sensor, error)
	// goverter:map Data | MapSensorData
	FromSqlSensorData(src *sqlc.SensorDatum) (*entities.SensorData, error)
	FromSqlSensorDataList(src []*sqlc.SensorDatum) ([]*entities.SensorData, error)
	FromDomainSensorData(src *entities.MqttPayload) *mqtt.MqttPayload
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
