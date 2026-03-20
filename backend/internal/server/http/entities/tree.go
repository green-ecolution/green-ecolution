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
	TreeClusterID  *int32                 `json:"tree_cluster_id"`
	PlantingYear   int32                  `json:"planting_year" validate:"required"`
	Species        string                 `json:"species"`
	Number         string                 `json:"number" validate:"required"`
	Latitude       float64                `json:"latitude" validate:"required,min=-90,max=90"`
	Longitude      float64                `json:"longitude" validate:"required,min=-180,max=180"`
	SensorID       *string                `json:"sensor_id"`
	Description    string                 `json:"description"`
	Provider       string                 `json:"provider"`
	AdditionalInfo map[string]interface{} `json:"additional_information"`
} //	@Name	TreeCreate

type TreeUpdateRequest struct {
	TreeClusterID  *int32                 `json:"tree_cluster_id"`
	PlantingYear   int32                  `json:"planting_year" validate:"gt=0"`
	Species        string                 `json:"species"`
	Number         string                 `json:"number"`
	Latitude       float64                `json:"latitude" validate:"omitempty,min=-90,max=90"`
	Longitude      float64                `json:"longitude" validate:"omitempty,min=-180,max=180"`
	SensorID       *string                `json:"sensor_id"`
	Description    string                 `json:"description"`
	Provider       string                 `json:"provider"`
	AdditionalInfo map[string]interface{} `json:"additional_information"`
} //	@Name	TreeUpdate

type TreeAddSensorRequest struct {
	SensorID *string `json:"sensor_id"`
} //	@Name	TreeAddSensor
