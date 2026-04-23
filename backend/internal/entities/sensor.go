package entities

import "time"

type SensorStatus string

const (
	SensorStatusOnline  SensorStatus = "online"
	SensorStatusOffline SensorStatus = "offline"
	SensorStatusUnknown SensorStatus = "unknown"
)

type Sensor struct {
	ID             SensorID
	CreatedAt      time.Time
	UpdatedAt      time.Time
	Status         SensorStatus
	LatestData     *SensorData
	Coordinate     Coordinate
	Provider       string
	AdditionalInfo map[string]interface{}
}

type SensorData struct {
	ID        int32
	SensorID  SensorID
	CreatedAt time.Time
	UpdatedAt time.Time
	Data      *MqttPayload
}

type SensorCreate struct {
	ID             SensorID
	Status         SensorStatus `validate:"oneof=online offline unknown"`
	LatestData     *SensorData
	Coordinate     Coordinate
	Provider       string
	AdditionalInfo map[string]interface{}
}

type SensorUpdate struct {
	Status         SensorStatus `validate:"oneof=online offline unknown"`
	LatestData     *SensorData
	Coordinate     Coordinate
	Provider       string
	AdditionalInfo map[string]interface{}
}
