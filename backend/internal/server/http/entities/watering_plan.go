package entities

import (
	"time"

	"github.com/google/uuid"
)

type WateringPlanStatus string //	@Name	WateringPlanStatus

const (
	WateringPlanStatusPlanned     WateringPlanStatus = "planned"
	WateringPlanStatusActive      WateringPlanStatus = "active"
	WateringPlanStatusCanceled    WateringPlanStatus = "canceled"
	WateringPlanStatusFinished    WateringPlanStatus = "finished"
	WateringPlanStatusNotCompeted WateringPlanStatus = "not competed"
	WateringPlanStatusUnknown     WateringPlanStatus = "unknown"
)

type WateringPlanResponse struct {
	ID                 int32                        `json:"id"`
	CreatedAt          time.Time                    `json:"created_at"`
	UpdatedAt          time.Time                    `json:"updated_at"`
	Date               time.Time                    `json:"date"`
	Description        string                       `json:"description"`
	Status             WateringPlanStatus           `json:"status"`
	Distance           *float64                     `json:"distance"`
	TotalWaterRequired *float64                     `json:"total_water_required"`
	UserIDs            []*uuid.UUID                 `json:"user_ids"`
	TreeClusters       []*TreeClusterInListResponse `json:"treeclusters"`
	Transporter        *VehicleResponse             `json:"transporter"`
	Trailer            *VehicleResponse             `json:"trailer" validate:"optional"`
	CancellationNote   string                       `json:"cancellation_note"`
	Evaluation         []*EvaluationValue           `json:"evaluation"`
	GpxURL             string                       `json:"gpx_url"`
	Duration           *float64                     `json:"duration"`
	RefillCount        int32                        `json:"refill_count"`
	Provider           string                       `json:"provider,omitempty"`
	AdditionalInfo     map[string]interface{}       `json:"additional_information,omitempty" validate:"optional"`
} //	@Name	WateringPlan

type WateringPlanInListResponse struct {
	ID                 int32                        `json:"id"`
	CreatedAt          time.Time                    `json:"created_at"`
	UpdatedAt          time.Time                    `json:"updated_at"`
	Date               time.Time                    `json:"date"`
	Description        string                       `json:"description"`
	Status             WateringPlanStatus           `json:"status"`
	Distance           *float64                     `json:"distance"`
	TotalWaterRequired *float64                     `json:"total_water_required"`
	UserIDs            []*uuid.UUID                 `json:"user_ids"`
	TreeClusters       []*TreeClusterInListResponse `json:"treeclusters"`
	Transporter        *VehicleResponse             `json:"transporter"`
	Trailer            *VehicleResponse             `json:"trailer" validate:"optional"`
	CancellationNote   string                       `json:"cancellation_note"`
	Provider           string                       `json:"provider,omitempty"`
	AdditionalInfo     map[string]interface{}       `json:"additional_information,omitempty" validate:"optional"`
} //	@Name	WateringPlanInList

type WateringPlanListResponse struct {
	Data       []*WateringPlanInListResponse `json:"data"`
	Pagination *Pagination                   `json:"pagination,omitempty" validate:"optional"`
} //	@Name	WateringPlanList

type WateringPlanCreateRequest struct {
	Date           time.Time              `json:"date" validate:"required,datetoday"`
	Description    string                 `json:"description"`
	TreeClusterIDs []*int32               `json:"tree_cluster_ids" validate:"required,min=1,dive,required"`
	TransporterID  *int32                 `json:"transporter_id" validate:"required"`
	TrailerID      *int32                 `json:"trailer_id"`
	UserIDs        []string               `json:"user_ids" validate:"required,min=1,dive,required"`
	Provider       string                 `json:"provider"`
	AdditionalInfo map[string]interface{} `json:"additional_information"`
} //	@Name	WateringPlanCreate

type WateringPlanUpdateRequest struct {
	Date             time.Time              `json:"date" validate:"required,datetoday"`
	Description      string                 `json:"description"`
	TreeClusterIDs   []*int32               `json:"tree_cluster_ids" validate:"required,min=1,dive,required"`
	TransporterID    *int32                 `json:"transporter_id" validate:"required"`
	TrailerID        *int32                 `json:"trailer_id"`
	UserIDs          []string               `json:"user_ids" validate:"required,min=1,dive,required"`
	CancellationNote string                 `json:"cancellation_note"`
	Status           WateringPlanStatus     `json:"status" validate:"oneof=planned active canceled finished 'not competed' unknown"`
	Evaluation       []*EvaluationValue     `json:"evaluation"`
	Provider         string                 `json:"provider"`
	AdditionalInfo   map[string]interface{} `json:"additional_information"`
} //	@Name	WateringPlanUpdate

type EvaluationValue struct {
	WateringPlanID int32    `json:"watering_plan_id"`
	TreeClusterID  int32    `json:"tree_cluster_id"`
	ConsumedWater  *float64 `json:"consumed_water"`
} //	@Name	EvaluationValue
