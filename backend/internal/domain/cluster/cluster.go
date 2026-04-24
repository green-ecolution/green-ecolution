package cluster

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

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
	WateringStatus shared.WateringStatus
	LastWatered    *time.Time
	MoistureLevel  float64
	RegionID       *int32
	Address        string
	Description    string
	Archived       bool
	Coordinate     *shared.Coordinate
	TreeIDs        []int32
	SoilCondition  TreeSoilCondition
	Name           string
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeClusterCreate struct {
	Address        string
	Description    string
	Name           string
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
	Name           string
	Provider       string
	AdditionalInfo map[string]interface{}
}

type TreeClusterQuery struct {
	WateringStatuses []shared.WateringStatus `query:"watering_statuses"`
	Regions          []string                `query:"regions"`
	IDs              []int32
	shared.Query
}
