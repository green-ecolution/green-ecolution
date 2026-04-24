package watering

import "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"

const EventTypeUpdate shared.EventType = "update watering plan"

type EventUpdate struct {
	shared.BasicEvent
	Prev *WateringPlan
	New  *WateringPlan
}

func NewEventUpdate(prev, newWp *WateringPlan) EventUpdate {
	return EventUpdate{
		BasicEvent: shared.NewBasicEvent(EventTypeUpdate),
		Prev:       prev,
		New:        newWp,
	}
}
