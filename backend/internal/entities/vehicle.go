package entities

import (
	"time"
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
	WaterCapacity  float64
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
	NumberPlate    string `validate:"required"`
	Description    string
	WaterCapacity  float64        `validate:"gt=0"`
	Status         VehicleStatus  `validate:"oneof=active available 'not available' unknown"`
	Type           VehicleType    `validate:"oneof=transporter trailer unknown"`
	Model          string         `validate:"required"`
	DrivingLicense DrivingLicense `validate:"oneof=B BE C CE"`
	Height         float64        `validate:"gt=0"`
	Width          float64        `validate:"gt=0"`
	Length         float64        `validate:"gt=0"`
	Weight         float64        `validate:"gt=0"`
	Provider       string
	AdditionalInfo map[string]interface{}
}

type VehicleUpdate struct {
	NumberPlate    string `validate:"required"`
	Description    string
	WaterCapacity  float64        `validate:"gt=0"`
	Status         VehicleStatus  `validate:"oneof=active available 'not available' unknown"`
	Type           VehicleType    `validate:"oneof=transporter trailer unknown"`
	Model          string         `validate:"required"`
	DrivingLicense DrivingLicense `validate:"oneof=B BE C CE"`
	Height         float64        `validate:"gt=0"`
	Width          float64        `validate:"gt=0"`
	Length         float64        `validate:"gt=0"`
	Weight         float64        `validate:"gt=0"`
	Provider       string
	AdditionalInfo map[string]interface{}
}

type VehicleQuery struct {
	Type         string `query:"type"`
	WithArchived bool   `query:"archived"`
	Query
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
