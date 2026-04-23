package entities

import (
	"time"
)

type SensorStatus string //	@Name	SensorStatus

const (
	SensorStatusOnline  SensorStatus = "online"
	SensorStatusOffline SensorStatus = "offline"
	SensorStatusUnknown SensorStatus = "unknown"
)

type SensorResponse struct {
	ID             string                 `json:"id"`
	CreatedAt      time.Time              `json:"created_at"`
	UpdatedAt      time.Time              `json:"updated_at"`
	Status         SensorStatus           `json:"status"`
	LatestData     *SensorDataResponse    `json:"latest_data"`
	Latitude       float64                `json:"latitude"`
	Longitude      float64                `json:"longitude"`
	Provider       string                 `json:"provider,omitempty"`
	AdditionalInfo map[string]interface{} `json:"additional_information,omitempty" validate:"optional"`
} //	@Name	Sensor

type SensorListResponse struct {
	Data       []*SensorResponse `json:"data"`
	Pagination *Pagination       `json:"pagination,omitempty" validate:"optional"`
} //	@Name	SensorList

type SensorDataResponse struct {
	CreatedAt   time.Time            `json:"created_at"`
	UpdatedAt   time.Time            `json:"updated_at"`
	Battery     float64              `json:"battery"`
	Humidity    float64              `json:"humidity"`
	Temperature float64              `json:"temperature"`
	Watermarks  []*WatermarkResponse `json:"watermarks"`
} //	@Name	SensorData

type SensorDataListResponse struct {
	Data []*SensorDataResponse `json:"data"`
} //	@Name	SensorDataList

type WatermarkResponse struct {
	Centibar   int `json:"centibar"`
	Resistance int `json:"resistance"`
	Depth      int `json:"depth"`
} //	@Name	WatermarkResponse
