package entities

import "time"

type TreeSoilCondition string

const (
	TreeSoilConditionSchluffig TreeSoilCondition = "schluffig"
	TreeSoilConditionSandig    TreeSoilCondition = "sandig"
	TreeSoilConditionLehmig    TreeSoilCondition = "lehmig"
	TreeSoilConditionTonig     TreeSoilCondition = "tonig"
	TreeSoilConditionUnknown   TreeSoilCondition = "unknown"
)

type TreeCluster struct {
	ID             int32
	CreatedAt      time.Time
	UpdatedAt      time.Time
	WateringStatus WateringStatus
	LastWatered    *time.Time
	MoistureLevel  float64
	Region         *Region
	Address        string
	Description    string
	Archived       bool
	Latitude       *float64
	Longitude      *float64
	Trees          []*Tree
	SoilCondition  TreeSoilCondition
	Name           string
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeClusterCreate struct {
	Address        string
	Description    string
	Name           string `validate:"required"`
	SoilCondition  TreeSoilCondition
	TreeIDs        []*int32
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeClusterUpdate struct {
	Address        string
	Description    string
	SoilCondition  TreeSoilCondition
	TreeIDs        []*int32
	Name           string `validate:"required"`
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeClusterQuery struct {
	WateringStatuses []WateringStatus `query:"watering_statuses"`
	Regions          []string         `query:"regions"`
	Query
}
