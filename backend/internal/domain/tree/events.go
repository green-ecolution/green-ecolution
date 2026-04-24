package tree

import "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"

const (
	EventTypeUpdate shared.EventType = "update tree"
	EventTypeCreate shared.EventType = "create tree"
	EventTypeDelete shared.EventType = "delete tree"
)

type EventUpdate struct {
	shared.BasicEvent
	Prev         *Tree
	New          *Tree
	PrevOfSensor *Tree
}

func NewEventUpdate(prev, newTree, prevOfSensor *Tree) EventUpdate {
	return EventUpdate{
		BasicEvent:   shared.NewBasicEvent(EventTypeUpdate),
		Prev:         prev,
		New:          newTree,
		PrevOfSensor: prevOfSensor,
	}
}

type EventCreate struct {
	shared.BasicEvent
	New          *Tree
	PrevOfSensor *Tree
}

func NewEventCreate(newTree, prevOfSensor *Tree) EventCreate {
	return EventCreate{
		BasicEvent:   shared.NewBasicEvent(EventTypeCreate),
		New:          newTree,
		PrevOfSensor: prevOfSensor,
	}
}

type EventDelete struct {
	shared.BasicEvent
	Prev *Tree
}

func NewEventDelete(prev *Tree) EventDelete {
	return EventDelete{
		BasicEvent: shared.NewBasicEvent(EventTypeDelete),
		Prev:       prev,
	}
}
