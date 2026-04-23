package entities

import (
	"errors"
	"slices"
	"time"
)

var (
	ErrSensorDataMalformed  = errors.New("sensor data must contain exactly 3 entries at depths 30, 60, and 90")
	ErrTreeBeyondMonitoring = errors.New("tree age exceeds monitored growth period")
	ErrNoSensorData         = errors.New("tree has no sensor assigned or sensor has no data")
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

var mapWateringStatus = map[int]WateringStatus{
	0: WateringStatusGood,
	1: WateringStatusModerate,
	2: WateringStatusBad,
}

func mapKpaRange(centibar, lower, higher int) int {
	if centibar < lower {
		return 0
	} else if centibar < higher {
		return 1
	} else {
		return 2
	}
}

func checkAndSortWatermarks(w []Watermark) (w30, w60, w90 Watermark, err error) {
	watermarks := slices.SortedFunc(slices.Values(w), func(a, b Watermark) int {
		return a.Depth - b.Depth
	})

	if len(watermarks) != 3 || watermarks[0].Depth != 30 || watermarks[1].Depth != 60 || watermarks[2].Depth != 90 {
		err = ErrSensorDataMalformed
		return
	}

	w30, w60, w90 = watermarks[0], watermarks[1], watermarks[2]
	return
}

func (t *Tree) CalculateWateringStatus(watermarks []Watermark) (WateringStatus, error) {
	currentYear := int32(time.Now().Year())
	treeLifetime := currentYear - t.PlantingYear.Year()
	w30, w60, w90, err := checkAndSortWatermarks(watermarks)
	if err != nil {
		return WateringStatusUnknown, err
	}

	statusList := make([]int, 3)
	const (
		lowerCentibarDefault  = 25
		higherCentibarDefault = 33

		lowerCentibarYear2Depth30  = 62
		higherCentibarYear2Depth30 = 81

		lowerCentibarYear3Depth30 = 1585
		lowerCentibarYear3Depth60 = 80
		lowerCentibarYear3Depth90 = 80
		noModerate                = -1
	)

	switch treeLifetime {
	case 0, 1:
		statusList[0] = mapKpaRange(w30.Centibar, lowerCentibarDefault, higherCentibarDefault)
		statusList[1] = mapKpaRange(w60.Centibar, lowerCentibarDefault, higherCentibarDefault)
		statusList[2] = mapKpaRange(w90.Centibar, lowerCentibarDefault, higherCentibarDefault)
	case 2:
		statusList[0] = mapKpaRange(w30.Centibar, lowerCentibarYear2Depth30, higherCentibarYear2Depth30)
		statusList[1] = mapKpaRange(w60.Centibar, lowerCentibarDefault, higherCentibarDefault)
		statusList[2] = mapKpaRange(w90.Centibar, lowerCentibarDefault, higherCentibarDefault)
	case 3:
		statusList[0] = mapKpaRange(w30.Centibar, lowerCentibarYear3Depth30, noModerate)
		statusList[1] = mapKpaRange(w60.Centibar, lowerCentibarYear3Depth60, noModerate)
		statusList[2] = mapKpaRange(w90.Centibar, lowerCentibarYear3Depth90, noModerate)
	default:
		return WateringStatusUnknown, ErrTreeBeyondMonitoring
	}

	slices.Sort(statusList)
	return mapWateringStatus[statusList[2]], nil
}

func (t *Tree) AssignSensor(sensor *Sensor) {
	t.Sensor = sensor
}

func (t *Tree) RemoveSensor() {
	t.Sensor = nil
	t.WateringStatus = WateringStatusUnknown
}

func (t *Tree) IsWateringStatusExpired(cutoff time.Time) bool {
	return t.WateringStatus == WateringStatusJustWatered && t.LastWatered != nil && t.LastWatered.Before(cutoff)
}

func (t *Tree) RefreshWateringStatus() (WateringStatus, bool, error) {
	if t.Sensor == nil || t.Sensor.LatestData == nil || t.Sensor.LatestData.Data == nil {
		return WateringStatusUnknown, false, ErrNoSensorData
	}

	status, err := t.CalculateWateringStatus(t.Sensor.LatestData.Data.Watermarks)
	if err != nil {
		return WateringStatusUnknown, false, err
	}

	return status, true, nil
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
