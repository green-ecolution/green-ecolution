package shared

type EventType string

type Event interface {
	Type() EventType
}

type BasicEvent struct {
	eventType EventType
}

func NewBasicEvent(t EventType) BasicEvent {
	return BasicEvent{eventType: t}
}

func (e BasicEvent) Type() EventType {
	return e.eventType
}
