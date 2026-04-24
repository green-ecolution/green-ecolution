package watering

import (
	"time"

	"github.com/google/uuid"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
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
	Distance           *shared.Distance
	TotalWaterRequired *float64
	UserIDs            []*uuid.UUID
	TreeClusterIDs     []int32
	TransporterID      *int32
	TrailerID          *int32
	CancellationNote   string
	Evaluation         []*EvaluationValue
	GpxURL             string
	RefillCount        int32
	Duration           time.Duration
	Provider           string
	AdditionalInfo     map[string]interface{}
}

const waterPerTree = 80.0

func CalculateRequiredWater(totalTreeCount int) float64 {
	return float64(totalTreeCount) * waterPerTree
}

func (wp *WateringPlan) ShouldRegenerateRoute(prev *WateringPlan) bool {
	if len(prev.TreeClusterIDs) != len(wp.TreeClusterIDs) {
		return true
	}
	if prev.TransporterID == nil || wp.TransporterID == nil {
		return prev.TransporterID != wp.TransporterID
	}
	if *prev.TransporterID != *wp.TransporterID {
		return true
	}
	if (prev.TrailerID == nil) != (wp.TrailerID == nil) {
		return true
	}
	if prev.TrailerID != nil && wp.TrailerID != nil && *prev.TrailerID != *wp.TrailerID {
		return true
	}
	for i, prevTcID := range prev.TreeClusterIDs {
		if prevTcID != wp.TreeClusterIDs[i] {
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
