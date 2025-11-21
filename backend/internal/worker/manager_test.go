package worker

import (
	"context"
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/stretchr/testify/assert"
)

const EventTypeTest entities.EventType = "test event"

type TestEvent struct {
	eventType entities.EventType
}

func (e TestEvent) Type() entities.EventType {
	return e.eventType
}

type TestSubscriber struct{}

func (s *TestSubscriber) EventType() entities.EventType {
	return EventTypeTest
}

func (s *TestSubscriber) HandleEvent(ctx context.Context, e entities.Event) error {
	_, ok := e.(TestEvent)
	if !ok {
		return errors.New("invalid event type")
	}
	return nil
}

func TestEventManager_Publish(t *testing.T) {
	t.Run("should publish an event", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)
		event := TestEvent{eventType: EventTypeTest}

		// when
		err := em.Publish(context.Background(), event)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return an error for unsupported event type", func(t *testing.T) {
		// given
		em := NewEventManager()
		event := TestEvent{eventType: EventTypeTest}

		// when
		err := em.Publish(context.Background(), event)

		// then
		assert.ErrorIs(t, err, ErrUnknownEventTypeErr)
	})
}

func TestEventManager_Subscribe(t *testing.T) {
	t.Run("should subscribe to an event type", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)

		// when
		id, ch, err := em.Subscribe(EventTypeTest)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, ch)
		assert.GreaterOrEqual(t, id, 0)
	})

	t.Run("should return an error for unsupported event type", func(t *testing.T) {
		// given
		em := NewEventManager()

		// when
		id, ch, err := em.Subscribe(EventTypeTest)

		// then
		assert.ErrorIs(t, err, ErrUnknownEventTypeErr)
		assert.Nil(t, ch)
		assert.Equal(t, -1, id)
	})
}

func TestEventManager_Unsubscribe(t *testing.T) {
	t.Run("should unsubscribe successfully", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)
		id, _, _ := em.Subscribe(EventTypeTest)

		// when
		err := em.Unsubscribe(EventTypeTest, id)

		// then
		assert.NoError(t, err)
	})

	t.Run("should return an error for invalid subscription ID", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)

		// when
		err := em.Unsubscribe(EventTypeTest, 42)

		// then
		assert.ErrorIs(t, err, ErrNotSubscribedErr)
	})
}

func TestEventManager_Run(t *testing.T) {
	t.Run("should process published events", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		go em.Run(ctx)

		id, ch, _ := em.Subscribe(EventTypeTest)
		event := TestEvent{eventType: EventTypeTest}

		// when
		_ = em.Publish(context.Background(), event)

		// then
		select {
		case receivedEvent := <-ch:
			assert.Equal(t, event, receivedEvent)
		case <-time.After(1 * time.Second):
			t.Fatal("event was not received")
		}

		_ = em.Unsubscribe(EventTypeTest, id)
	})
}

func TestEventManager_RunSubscription(t *testing.T) {
	t.Run("should handle events via subscriber", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		subscriber := &TestSubscriber{}

		go func() {
			err := em.RunSubscription(ctx, subscriber)
			assert.NoError(t, err)
		}()

		event := TestEvent{eventType: EventTypeTest}
		_ = em.Publish(context.Background(), event)

		// Simulate some processing time
		time.Sleep(100 * time.Millisecond)
	})
}

func TestEventManager_PublishEventsInLoop(t *testing.T) {
	t.Run("should publish events in loop", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)
		ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
		defer cancel()
		go em.Run(ctx)

		id, ch, _ := em.Subscribe(EventTypeTest)
		receivedCount := 0
		go func() {
			for {
				select {
				case <-ctx.Done():
					return
				case <-ch:
					fmt.Println("received event. eventcount: ", receivedCount)
					receivedCount++
				}

			}
		}()

		// when
		for i := 0; i < 100; i++ {
			fmt.Println("Publish event", i)
			event := TestEvent{eventType: EventTypeTest}
			err := em.Publish(context.Background(), event)
			assert.NoError(t, err)
		}

		// then
		<-ctx.Done()
		_ = em.Unsubscribe(EventTypeTest, id)
		assert.Equal(t, 100, receivedCount)
	})

	t.Run("should buffer 100 if not consumed in channel", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)
		ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
		defer cancel()
		go em.Run(ctx)

		_, _, _ = em.Subscribe(EventTypeTest)

		// when
		for i := 0; i < 101; i++ {
			fmt.Println("Publish event", i)
			event := TestEvent{eventType: EventTypeTest}
			err := em.Publish(context.Background(), event)
			assert.NoError(t, err)
		}

		// then
		<-ctx.Done()
		assert.Equal(t, 100, len(em.eventCh))
	})

	t.Run("should error on buffered 101 if not consumed in channel and context timeout", func(t *testing.T) {
		// given
		em := NewEventManager(EventTypeTest)
		ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
		defer cancel()
		go em.Run(ctx)

		_, _, _ = em.Subscribe(EventTypeTest)

		// when
		for i := 0; i < 102; i++ {
			fmt.Println("Publish event", i)
			event := TestEvent{eventType: EventTypeTest}
			err := em.Publish(ctx, event)
			if i == 101 {
				assert.Error(t, err)
				assert.ErrorIs(t, err, context.DeadlineExceeded)
			} else {
				assert.NoError(t, err)
			}
		}

		// then
		<-ctx.Done()
	})
}
