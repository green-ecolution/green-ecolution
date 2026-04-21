package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

func SensorFromResponse(source *domain.Sensor) *entities.SensorResponse {
	if source == nil {
		return nil
	}
	return &entities.SensorResponse{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		Status:         MapSensorStatus(source.Status),
		LatestData:     MapLatestDataToResponse(source.LatestData),
		Latitude:       source.Latitude,
		Longitude:      source.Longitude,
		Provider:       source.Provider,
		AdditionalInfo: source.AdditionalInfo,
	}
}

func SensorFromDataResponse(source *domain.SensorData) *entities.SensorDataResponse {
	return MapLatestDataToResponse(source)
}

func SensorFromWatermarkResponse(source *domain.Watermark) *entities.WatermarkResponse {
	if source == nil {
		return nil
	}
	return &entities.WatermarkResponse{
		Centibar:   source.Centibar,
		Resistance: source.Resistance,
		Depth:      source.Depth,
	}
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
