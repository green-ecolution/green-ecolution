package entities

import (
	"time"
)

type TreeResponse struct {
	ID             int32                  `json:"id"`
	CreatedAt      time.Time              `json:"created_at"`
	UpdatedAt      time.Time              `json:"updated_at"`
	TreeClusterID  *int32                 `json:"tree_cluster_id" validate:"optional"`
	Sensor         *SensorResponse        `json:"sensor" validate:"optional"`
	LastWatered    *time.Time             `json:"last_watered,omitempty" validate:"optional"`
	PlantingYear   int32                  `json:"planting_year"`
	Species        string                 `json:"species"`
	Number         string                 `json:"number"`
	Latitude       float64                `json:"latitude"`
	Longitude      float64                `json:"longitude"`
	WateringStatus WateringStatus         `json:"watering_status"`
	Description    string                 `json:"description"`
	Provider       string                 `json:"provider,omitempty"`
	AdditionalInfo map[string]interface{} `json:"additional_information,omitempty" validate:"optional"`
} //	@Name	Tree

type TreeListResponse struct {
	Data       []*TreeResponse `json:"data"`
	Pagination *Pagination     `json:"pagination,omitempty" validate:"optional"`
} //	@Name	TreeList

type TreeCreateRequest struct {
	TreeClusterID  *int32                 `json:"tree_cluster_id" validate:"optional"`
	PlantingYear   int32                  `json:"planting_year"`
	Species        string                 `json:"species"`
	Number         string                 `json:"number"`
	Latitude       float64                `json:"latitude"`
	Longitude      float64                `json:"longitude"`
	SensorID       *string                `json:"sensor_id" validate:"optional"`
	Description    string                 `json:"description"`
	Provider       string                 `json:"provider" validate:"optional"`
	AdditionalInfo map[string]interface{} `json:"additional_information" validate:"optional"`
} //	@Name	TreeCreate

type TreeUpdateRequest struct {
	TreeClusterID  *int32                 `json:"tree_cluster_id" validate:"optional"`
	PlantingYear   int32                  `json:"planting_year"`
	Species        string                 `json:"species"`
	Number         string                 `json:"number"`
	Latitude       float64                `json:"latitude"`
	Longitude      float64                `json:"longitude"`
	SensorID       *string                `json:"sensor_id" validate:"optional"`
	Description    string                 `json:"description"`
	Provider       string                 `json:"provider" validate:"optional"`
	AdditionalInfo map[string]interface{} `json:"additional_information" validate:"optional"`
} //	@Name	TreeUpdate

type TreeAddSensorRequest struct {
	SensorID *string `json:"sensor_id"`
} //	@Name	TreeAddSensor
