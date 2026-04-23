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
	PlantingYear   PlantingYear
	Species        string
	Number         string
	Coordinate     Coordinate
	WateringStatus WateringStatus
	Description    string
	LastWatered    *time.Time
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeCreate struct {
	TreeClusterID  *int32
	SensorID       *SensorID
	PlantingYear   PlantingYear
	Species        string
	Number         string
	Coordinate     Coordinate
	Description    string
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeUpdate struct {
	TreeClusterID  *int32
	SensorID       *SensorID
	PlantingYear   PlantingYear
	Species        string
	Number         string
	Coordinate     Coordinate
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

type TreeWithDistance struct {
	Tree     *Tree
	Distance Distance
}
