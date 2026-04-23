package shared

import (
	"slices"
	"time"
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
	WateringStatus WateringStatus
	LastWatered    *time.Time
	MoistureLevel  float64
	Region         *Region
	Address        string
	Description    string
	Archived       bool
	Coordinate     *Coordinate
	Trees          []*Tree
	SoilCondition  TreeSoilCondition
	Name           string
	Provider       string
	AdditionalInfo map[string]interface{}
}

func (tc *TreeCluster) CalculateWateringStatus(sensorData []*SensorData) (WateringStatus, error) {
	if len(sensorData) == 0 {
		return WateringStatusUnknown, nil
	}

	youngest := tc.YoungestTree()
	if youngest == nil {
		return WateringStatusUnknown, nil
	}

	watermarks, err := tc.AverageWatermarks(sensorData)
	if err != nil {
		return WateringStatusUnknown, err
	}

	return youngest.CalculateWateringStatus(watermarks)
}

func (tc *TreeCluster) YoungestTree() *Tree {
	sortedTrees := slices.SortedFunc(slices.Values(tc.Trees), func(a, b *Tree) int {
		return int(a.PlantingYear.Year() - b.PlantingYear.Year())
	})

	if len(sortedTrees) > 0 {
		return sortedTrees[0]
	}

	return nil
}

func (tc *TreeCluster) AverageWatermarks(sensorData []*SensorData) ([]Watermark, error) {
	var w30CentibarAvg, w60CentibarAvg, w90CentibarAvg int
	for _, data := range sensorData {
		w30, w60, w90, err := checkAndSortWatermarks(data.Data.Watermarks)
		if err != nil {
			return nil, ErrSensorDataMalformed
		}

		w30CentibarAvg += w30.Centibar
		w60CentibarAvg += w60.Centibar
		w90CentibarAvg += w90.Centibar
	}

	return []Watermark{
		{
			Centibar: w30CentibarAvg / len(sensorData),
			Depth:    30,
		},
		{
			Centibar: w60CentibarAvg / len(sensorData),
			Depth:    60,
		},
		{
			Centibar: w90CentibarAvg / len(sensorData),
			Depth:    90,
		},
	}, nil
}

func (tc *TreeCluster) NeedsPositionUpdate(prevTree, newTree *Tree) bool {
	if prevTree.Coordinate != newTree.Coordinate {
		return true
	}
	if prevTree.TreeCluster == nil || newTree.TreeCluster == nil {
		return prevTree.TreeCluster != newTree.TreeCluster
	}
	if prevTree.TreeCluster.ID != newTree.TreeCluster.ID {
		return true
	}
	if prevTree.Sensor != newTree.Sensor {
		return true
	}
	return false
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
	WateringStatuses []WateringStatus `query:"watering_statuses"`
	Regions          []string         `query:"regions"`
	Query
}
