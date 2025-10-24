package entities

import "time"

type SensorStatus string

const (
	SensorStatusOnline  SensorStatus = "online"
	SensorStatusOffline SensorStatus = "offline"
	SensorStatusUnknown SensorStatus = "unknown"
)

type Sensor struct {
	ID             string
	CreatedAt      time.Time
	UpdatedAt      time.Time
	Status         SensorStatus
	LatestData     *SensorData
	Latitude       float64
	Longitude      float64
	Provider       string
	AdditionalInfo map[string]interface{}
}

type SensorData struct {
	ID        int32
	SensorID  string
	CreatedAt time.Time
	UpdatedAt time.Time
	Data      *MqttPayload
}

type SensorCreate struct {
	ID             string       `validate:"required"`
	Status         SensorStatus `validate:"oneof=online offline unknown"`
	LatestData     *SensorData
	Latitude       float64 `validate:"required,max=90,min=-90"`
	Longitude      float64 `validate:"required,max=180,min=-180"`
	Provider       string
	AdditionalInfo map[string]interface{}
}

type SensorUpdate struct {
	Status         SensorStatus `validate:"oneof=online offline unknown"`
	LatestData     *SensorData
	Latitude       float64 `validate:"required,max=90,min=-90"`
	Longitude      float64 `validate:"required,max=180,min=-180"`
	Provider       string
	AdditionalInfo map[string]interface{}
}
