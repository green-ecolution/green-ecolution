package sensor

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func ToResponse(src *domain.MqttPayload) *MqttPayloadResponse {
	if src == nil {
		return nil
	}
	resp := &MqttPayloadResponse{
		Device:      src.Device,
		Battery:     src.Battery,
		Humidity:    src.Humidity,
		Temperature: src.Temperature,
		Latitude:    src.Latitude,
		Longitude:   src.Longitude,
	}
	if src.Watermarks != nil {
		resp.Watermarks = make([]WatermarkResponse, len(src.Watermarks))
		for i, w := range src.Watermarks {
			resp.Watermarks[i] = WatermarkResponse{
				Resistance: w.Resistance,
				Centibar:   w.Centibar,
				Depth:      w.Depth,
			}
		}
	}
	return resp
}

func ToResponseList(src []*domain.MqttPayload) []*MqttPayloadResponse {
	return utils.MapSlice(src, ToResponse)
}

func FromResponse(src *MqttPayloadResponse) *domain.MqttPayload {
	if src == nil {
		return nil
	}
	payload := &domain.MqttPayload{
		Device:      src.Device,
		Battery:     src.Battery,
		Humidity:    src.Humidity,
		Temperature: src.Temperature,
		Latitude:    src.Latitude,
		Longitude:   src.Longitude,
	}
	if src.Watermarks != nil {
		payload.Watermarks = make([]domain.Watermark, len(src.Watermarks))
		for i, w := range src.Watermarks {
			payload.Watermarks[i] = domain.Watermark{
				Resistance: w.Resistance,
				Centibar:   w.Centibar,
				Depth:      w.Depth,
			}
		}
	}
	return payload
}

func FromResponseList(src []*MqttPayloadResponse) []*domain.MqttPayload {
	return utils.MapSlice(src, FromResponse)
}
