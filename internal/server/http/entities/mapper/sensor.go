package mapper

import (
	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/backend/internal/utils:MapKeyValueInterface
// goverter:extend MapSensorStatus MapLatestDataToResponse
type SensorHTTPMapper interface {
	FromResponse(src *domain.Sensor) *entities.SensorResponse
	FromDataResponse(src *domain.SensorData) *entities.SensorDataResponse
	FromWatermarkResponse(src *domain.Watermark) *entities.WatermarkResponse
}

func MapLatestDataToResponse(sensorData *domain.SensorData) *entities.SensorDataResponse {
	if sensorData == nil || sensorData.Data == nil {
		return nil
	}

	return &entities.SensorDataResponse{
		CreatedAt:   sensorData.CreatedAt,
		UpdatedAt:   sensorData.UpdatedAt,
		Battery:     sensorData.Data.Battery,
		Humidity:    sensorData.Data.Humidity,
		Temperature: sensorData.Data.Temperature,
		Watermarks:  mapWatermarkData(sensorData.Data.Watermarks),
	}
}

func mapWatermarkData(watermarks []domain.Watermark) []*entities.WatermarkResponse {
	responses := make([]*entities.WatermarkResponse, len(watermarks))
	for i, w := range watermarks {
		responses[i] = &entities.WatermarkResponse{
			Centibar:   w.Centibar,
			Resistance: w.Resistance,
			Depth:      w.Depth,
		}
	}
	return responses
}

func MapSensorStatus(src domain.SensorStatus) entities.SensorStatus {
	return entities.SensorStatus(src)
}
