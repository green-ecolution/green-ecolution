package sensor

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func ToResponse(src *sensor.MqttPayload) *MqttPayloadResponse {
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

func ToResponseList(src []*sensor.MqttPayload) []*MqttPayloadResponse {
	return utils.MapSlice(src, ToResponse)
}

func FromResponse(src *MqttPayloadResponse) *sensor.MqttPayload {
	if src == nil {
		return nil
	}
	payload := &sensor.MqttPayload{
		Device:      src.Device,
		Battery:     src.Battery,
		Humidity:    src.Humidity,
		Temperature: src.Temperature,
		Latitude:    src.Latitude,
		Longitude:   src.Longitude,
	}
	if src.Watermarks != nil {
		payload.Watermarks = make([]sensor.Watermark, len(src.Watermarks))
		for i, w := range src.Watermarks {
			payload.Watermarks[i] = sensor.Watermark{
				Resistance: w.Resistance,
				Centibar:   w.Centibar,
				Depth:      w.Depth,
			}
		}
	}
	return payload
}

func FromResponseList(src []*MqttPayloadResponse) []*sensor.MqttPayload {
	return utils.MapSlice(src, FromResponse)
}
