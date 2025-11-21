// Package worker implements an event management system for publishing and subscribing to events.
package worker

import (
	"context"
	"errors"
	"log/slog"
	"sync"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
)

var (
	ErrUnknownEventTypeErr = errors.New("unsupported event type")
	ErrNotSubscribedErr    = errors.New("not subscribed")
)

const eventChSize = 100

// Subscriber defines the interface for handling events of a specific type.
type Subscriber interface {
	// HandleEvent processes the received event.
	HandleEvent(ctx context.Context, event entities.Event) error

	// EventType returns the type of events this subscriber is interested in.
	EventType() entities.EventType
}

// EventManager manages event publication and subscription.
type EventManager struct {
	eventCh    chan entities.Event
	subscriber map[entities.EventType]map[int]chan<- entities.Event
	nextID     int
	eventTypes map[entities.EventType]struct{}
	rwMutex    sync.RWMutex
}

// NewEventManager creates a new EventManager for the given event types.
//
// The EventManager facilitates the publication and subscription of events. It ensures
// that only supported event types are processed and provides thread-safe methods for managing
// subscriptions and event delivery.
//
// Parameters:
// - eventTypes: A variadic list of entities.EventType values that the manager should support.
//
// Returns:
// - A pointer to the newly created EventManager instance.
//
// Example usage:
//
//	em := NewEventManager(
//		entities.EventType("UserCreated"),
//		entities.EventType("UserDeleted"),
//	)
func NewEventManager(eventTypes ...entities.EventType) *EventManager {
	eventTypeMap := make(map[entities.EventType]struct{})
	for _, eventType := range eventTypes {
		eventTypeMap[eventType] = struct{}{}
	}

	subscriber := make(map[entities.EventType]map[int]chan<- entities.Event)
	for eventType := range eventTypeMap {
		subscriber[eventType] = make(map[int]chan<- entities.Event)
	}

	return &EventManager{
		eventCh:    make(chan entities.Event, eventChSize),
		subscriber: subscriber,
		nextID:     0,
		eventTypes: eventTypeMap,
	}
}

// Publish sends an event to all subscribers of its type.
//
// This method ensures that the event is only published if its type is supported by the EventManager.
// Events are delivered asynchronously to prevent blocking the publisher.
//
// Parameters:
// - event: The event to be published. Must implement the entities.Event interface.
//
// Returns:
// - An error if the event type is unsupported.
//
// Example usage:
//
//	err := em.Publish(myEvent)
//	if err != nil {
//		log.Fatalf("Failed to publish event: %v", err)
//	}
func (e *EventManager) Publish(ctx context.Context, event entities.Event) error {
	if _, ok := e.eventTypes[event.Type()]; !ok {
		return ErrUnknownEventTypeErr
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case e.eventCh <- event:
		return nil
	}
}

// Subscribe registers a new subscription for the specified event type.
//
// A subscription allows a caller to receive events of a specific type via a dedicated channel.
// The caller must manage the lifecycle of the channel.
//
// Parameters:
// - eventType: The type of events to subscribe to.
//
// Returns:
// - A unique subscription ID.
// - A channel to receive events of the specified type.
// - An error if the event type is unsupported.
//
// Example usage:
//
//	id, ch, err := em.Subscribe(entities.EventType("UserCreated"))
//	if err != nil {
//		log.Fatalf("Failed to subscribe: %v", err)
//	}
func (e *EventManager) Subscribe(eventType entities.EventType) (id int, ch <-chan entities.Event, err error) {
	if _, ok := e.eventTypes[eventType]; !ok {
		return -1, nil, ErrUnknownEventTypeErr
	}

	e.rwMutex.Lock()
	defer e.rwMutex.Unlock()

	channel := make(chan entities.Event)
	subID := e.nextID
	e.subscriber[eventType][id] = channel
	e.nextID++

	slog.Info("start to subscribe an event", "event_type", eventType, "event_id", subID)
	return subID, channel, nil
}

// Unsubscribe removes a subscription identified by its event type and ID.
//
// This method ensures that the channel associated with the subscription is closed
// and the subscription is removed from the internal registry.
//
// Parameters:
// - eventType: The type of the event subscription.
// - id: The unique ID of the subscription to be removed.
//
// Returns:
// - An error if the event type is unsupported or the subscription ID is not found.
//
// Example usage:
//
//	err := em.Unsubscribe(entities.EventType("UserCreated"), subscriptionID)
//	if err != nil {
//		log.Printf("Failed to unsubscribe: %v", err)
//	}
func (e *EventManager) Unsubscribe(eventType entities.EventType, id int) error {
	if _, ok := e.eventTypes[eventType]; !ok {
		return ErrUnknownEventTypeErr
	}

	e.rwMutex.Lock()
	defer e.rwMutex.Unlock()

	slog.Info("unsubscribe to an event", "event_type", eventType, "event_id", id)
	return e.unsubscribe(eventType, id)
}

// unsubscribe is an internal method to remove a subscription and close its channel.
func (e *EventManager) unsubscribe(eventType entities.EventType, id int) error {
	ch, ok := e.subscriber[eventType][id]
	if !ok {
		return ErrNotSubscribedErr
	}

	close(ch)
	delete(e.subscriber[eventType], id)

	return nil
}

// cleanup removes all subscriptions and clears the event types.
//
// This method is called internally to release resources and ensure that all channels are closed.
func (e *EventManager) cleanup() {
	slog.Info("cleanup event manager")
	e.rwMutex.Lock()
	defer e.rwMutex.Unlock()

	for eventType, subscriber := range e.subscriber {
		for id := range subscriber {
			_ = e.unsubscribe(eventType, id)
		}

		delete(e.eventTypes, eventType)
	}
}

// Stop stops the EventManager by closing its event channel.
//
// This method should be called to gracefully terminate the EventManager's operations.
func (e *EventManager) Stop() {
	close(e.eventCh)
}

// Run processes events and sends them to appropriate subscribers.
//
// This is a blocking method and should be run in a separate goroutine.
// It listens for published events and distributes them to subscribers of matching types.
//
// Parameters:
// - ctx: A context to control the lifecycle of the Run method. Canceling the context will stop event processing.
//
// Example usage:
//
//	ctx, cancel := context.WithCancel(context.Background())
//	go em.Run(ctx)
//	// Perform operations...
//	cancel()
func (e *EventManager) Run(ctx context.Context) {
	slog.Info("starting event manager")
	defer e.cleanup()
	for {
		select {
		case <-ctx.Done():
			return
		case v, ok := <-e.eventCh:
			if !ok {
				return
			}

			// TODO: Maybe unsubscribe if can't write
			e.rwMutex.RLock()
			for _, ch := range e.subscriber[v.Type()] {
				select {
				case <-ctx.Done():
					e.rwMutex.RUnlock()
					return
				case ch <- v:
				}
			}
			e.rwMutex.RUnlock()
		}
	}
}

// RunSubscription manages a single subscription, forwarding events to the Subscriber.
//
// This is a blocking method and should be run in a separate goroutine.
// It allows a Subscriber implementation to process events in its own context.
// It ensures that the subscription is cleaned up when the context is canceled or an error occurs.
//
// Parameters:
// - ctx: A context to control the lifecycle of the subscription.
// - sub: The Subscriber implementation to handle events of a specific type.
//
// Returns:
// - An error if event handling fails or the subscription encounters an issue.
//
// Example usage:
//
//	subscriber := MySubscriber{}
//	go func() {
//		err := em.RunSubscription(ctx, subscriber)
//		if err != nil {
//			log.Printf("Subscription error: %v", err)
//		}
//	}()
func (e *EventManager) RunSubscription(ctx context.Context, sub Subscriber) error {
	id, ch, err := e.Subscribe(sub.EventType())
	if err != nil {
		return err
	}
	defer func() {
		_ = e.Unsubscribe(sub.EventType(), id)
	}()
	for {
		select {
		case <-ctx.Done():
			return nil
		case v, ok := <-ch:
			if !ok {
				return nil
			}
			if err := sub.HandleEvent(ctx, v); err != nil {
				return err
			}
		}
	}
}
