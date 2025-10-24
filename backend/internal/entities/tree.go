package entities

import (
	"time"
)

type Tree struct {
	ID             int32
	CreatedAt      time.Time
	UpdatedAt      time.Time
	TreeCluster    *TreeCluster
	Sensor         *Sensor
	PlantingYear   int32
	Species        string
	Number         string
	Latitude       float64
	Longitude      float64
	WateringStatus WateringStatus
	Description    string
	LastWatered    *time.Time
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeCreate struct {
	TreeClusterID  *int32
	SensorID       *string
	PlantingYear   int32 `validate:"required"`
	Species        string
	Number         string  `validate:"required"`
	Latitude       float64 `validate:"required,max=90,min=-90"`
	Longitude      float64 `validate:"required,max=180,min=-180"`
	Description    string
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeUpdate struct {
	TreeClusterID  *int32
	SensorID       *string
	PlantingYear   int32 `validate:"gt=0"`
	Species        string
	Number         string
	Latitude       float64 `validate:"omitempty,min=-90,max=90"`
	Longitude      float64 `validate:"omitempty,min=-180,max=180"`
	Description    string
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeQuery struct {
	WateringStatuses []WateringStatus `query:"watering_statuses"`
	HasCluster       *bool            `query:"has_cluster"`
	PlantingYears    []int32          `query:"planting_years"`
	Query
}
