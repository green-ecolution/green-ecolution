package cluster

import "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"

const EventTypeUpdate shared.EventType = "update tree cluster"

type EventUpdate struct {
	shared.BasicEvent
	Prev *TreeCluster
	New  *TreeCluster
}

func NewEventUpdate(prev, newTc *TreeCluster) EventUpdate {
	return EventUpdate{
		BasicEvent: shared.NewBasicEvent(EventTypeUpdate),
		Prev:       prev,
		New:        newTc,
	}
}
