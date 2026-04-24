package vehicle

import (
	"fmt"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type VehicleStatus string

const (
	VehicleStatusActive       VehicleStatus = "active"
	VehicleStatusAvailable    VehicleStatus = "available"
	VehicleStatusNotAvailable VehicleStatus = "not available"
	VehicleStatusUnknown      VehicleStatus = "unknown"
)

type VehicleType string

const (
	VehicleTypeTransporter VehicleType = "transporter"
	VehicleTypeTrailer     VehicleType = "trailer"
	VehicleTypeUnknown     VehicleType = "unknown"
)

type Vehicle struct {
	ID             int32
	CreatedAt      time.Time
	UpdatedAt      time.Time
	ArchivedAt     time.Time
	NumberPlate    string
	Description    string
	WaterCapacity  shared.WaterCapacity
	Status         VehicleStatus
	Type           VehicleType
	Model          string
	DrivingLicense DrivingLicense
	Height         float64
	Width          float64
	Length         float64
	Weight         float64
	Provider       string
	AdditionalInfo map[string]interface{}
}

type VehicleCreate struct {
	NumberPlate    string
	Description    string
	WaterCapacity  shared.WaterCapacity
	Status         VehicleStatus
	Type           VehicleType
	Model          string
	DrivingLicense DrivingLicense
	Height         float64
	Width          float64
	Length         float64
	Weight         float64
	Provider       string
	AdditionalInfo map[string]interface{}
}

type VehicleUpdate struct {
	NumberPlate    string
	Description    string
	WaterCapacity  shared.WaterCapacity
	Status         VehicleStatus
	Type           VehicleType
	Model          string
	DrivingLicense DrivingLicense
	Height         float64
	Width          float64
	Length         float64
	Weight         float64
	Provider       string
	AdditionalInfo map[string]interface{}
}

type VehicleQuery struct {
	Type         VehicleType `query:"type"`
	WithArchived bool        `query:"archived"`
	OnlyArchived bool        `query:"only_archived"`
	shared.Query
}

func MergeVehicles(transporter, trailer *Vehicle) *Vehicle {
	if transporter == nil {
		return nil
	}
	if trailer == nil {
		return transporter
	}

	width := transporter.Width
	if trailer.Width > width {
		width = trailer.Width
	}

	height := transporter.Height
	if trailer.Height > height {
		height = trailer.Height
	}

	return &Vehicle{
		Width:         width,
		Height:        height,
		Length:        transporter.Length + trailer.Length,
		Weight:        transporter.Weight + trailer.Weight,
		WaterCapacity: transporter.WaterCapacity.Add(trailer.WaterCapacity),
		Type:          VehicleTypeTransporter,
		NumberPlate:   fmt.Sprintf("%s - %s", transporter.NumberPlate, trailer.NumberPlate),
	}
}

func ParseVehicleType(vehicleTypeStr string) VehicleType {
	switch vehicleTypeStr {
	case string(VehicleTypeTrailer):
		return VehicleTypeTrailer
	case string(VehicleTypeTransporter):
		return VehicleTypeTransporter
	default:
		return VehicleTypeUnknown
	}
}
