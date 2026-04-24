package sensor

import "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"

const EventTypeNewData shared.EventType = "receive sensor data"

type EventNewData struct {
	shared.BasicEvent
	New *SensorData
}

func NewEventNewData(newData *SensorData) EventNewData {
	return EventNewData{
		BasicEvent: shared.NewBasicEvent(EventTypeNewData),
		New:        newData,
	}
}
