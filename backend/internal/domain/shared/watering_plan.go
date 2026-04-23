package entities

import (
	"time"

	"github.com/google/uuid"
)

type WateringPlanStatus string

const (
	WateringPlanStatusPlanned     WateringPlanStatus = "planned"
	WateringPlanStatusActive      WateringPlanStatus = "active"
	WateringPlanStatusCanceled    WateringPlanStatus = "canceled"
	WateringPlanStatusFinished    WateringPlanStatus = "finished"
	WateringPlanStatusNotCompeted WateringPlanStatus = "not competed"
	WateringPlanStatusUnknown     WateringPlanStatus = "unknown"
)

type WateringPlan struct {
	ID                 int32
	CreatedAt          time.Time
	UpdatedAt          time.Time
	Date               time.Time
	Description        string
	Status             WateringPlanStatus
	Distance           *Distance
	TotalWaterRequired *float64
	UserIDs            []*uuid.UUID
	TreeClusters       []*TreeCluster
	Transporter        *Vehicle
	Trailer            *Vehicle
	CancellationNote   string
	Evaluation         []*EvaluationValue
	GpxURL             string
	RefillCount        int32
	Duration           time.Duration
	Provider           string
	AdditionalInfo     map[string]interface{}
}

const waterPerTree = 80.0

func (wp *WateringPlan) CalculateRequiredWater() float64 {
	var total float64
	for _, tc := range wp.TreeClusters {
		total += float64(len(tc.Trees)) * waterPerTree
	}
	return total
}

func (wp *WateringPlan) ShouldRegenerateRoute(prev *WateringPlan) bool {
	if len(prev.TreeClusters) != len(wp.TreeClusters) {
		return true
	}
	if prev.Transporter.ID != wp.Transporter.ID {
		return true
	}
	if (prev.Trailer == nil) != (wp.Trailer == nil) {
		return true
	}
	if prev.Trailer != nil && wp.Trailer != nil && prev.Trailer.ID != wp.Trailer.ID {
		return true
	}
	for i, prevTc := range prev.TreeClusters {
		if prevTc.ID != wp.TreeClusters[i].ID {
			return true
		}
	}
	return false
}

func (wp *WateringPlan) IsExpired(cutoff time.Time) bool {
	return (wp.Status == WateringPlanStatusActive ||
		wp.Status == WateringPlanStatusPlanned ||
		wp.Status == WateringPlanStatusUnknown) &&
		wp.Date.Before(cutoff)
}

type WateringPlanCreate struct {
	Date           time.Time
	Description    string
	TreeClusterIDs []*int32
	TransporterID  *int32
	TrailerID      *int32
	UserIDs        []*uuid.UUID
	Provider       string
	AdditionalInfo map[string]interface{}
}

type WateringPlanUpdate struct {
	Date             time.Time
	Description      string
	TreeClusterIDs   []*int32
	TransporterID    *int32
	TrailerID        *int32
	CancellationNote string
	Status           WateringPlanStatus
	Evaluation       []*EvaluationValue
	UserIDs          []*uuid.UUID
	Provider         string
	AdditionalInfo   map[string]interface{}
}

type EvaluationValue struct {
	WateringPlanID int32
	TreeClusterID  int32
	ConsumedWater  *float64
}
