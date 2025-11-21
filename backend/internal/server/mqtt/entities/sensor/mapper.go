package sensor

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTime
type MqttMqttMapper interface {
	ToResponse(src *domain.MqttPayload) *MqttPayloadResponse
	ToResponseList(src []*domain.MqttPayload) []*MqttPayloadResponse
	FromResponse(src *MqttPayloadResponse) *domain.MqttPayload
	FromResponseList(src []*MqttPayloadResponse) []*domain.MqttPayload
}
