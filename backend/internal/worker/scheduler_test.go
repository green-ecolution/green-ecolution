package worker

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

type TestWork struct {
	callCount int
}

func (t *TestWork) Do(ctx context.Context) error {
	t.callCount++
	return nil
}

type ErrorWork struct {
	callCount int
}

func (e *ErrorWork) Do(ctx context.Context) error {
	e.callCount++
	return errors.New("intentional error")
}

func TestSchedulerWithStruct(t *testing.T) {
	t.Run("should create a scheduler instance and execute Do() at least once", func(t *testing.T) {
		// given
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		task := &TestWork{}

		// when
		scheduler := NewScheduler(50*time.Millisecond, task)
		go scheduler.Run(ctx)

		time.Sleep(150 * time.Millisecond)
		cancel()

		// then
		assert.GreaterOrEqual(t, task.callCount, 1, "Scheduler should execute Do() at least once")
	})

	t.Run("should stop execution when Stop() is called", func(t *testing.T) {
		// given
		ctx, cancel := context.WithCancel(context.Background())
		task := &TestWork{}

		// when
		scheduler := NewScheduler(50*time.Millisecond, task)
		go scheduler.Run(ctx)

		time.Sleep(150 * time.Millisecond)
		cancel()
		countAfterStop := task.callCount

		time.Sleep(150 * time.Millisecond)

		// then
		assert.Equal(t, countAfterStop, task.callCount, "Scheduler should stop executing Do() after Stop() is called")
	})

	t.Run("should continue execution even if Do() returns an error", func(t *testing.T) {
		// given
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		task := &ErrorWork{}

		// when
		scheduler := NewScheduler(50*time.Millisecond, task)
		go scheduler.Run(ctx)

		time.Sleep(150 * time.Millisecond)
		cancel()

		// then
		assert.GreaterOrEqual(t, task.callCount, 1, "Scheduler should continue running even if Do() returns an error")
	})

	t.Run("should not execute any Do() calls if started and stopped immediately", func(t *testing.T) {
		// given
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		task := &TestWork{}

		// when
		scheduler := NewScheduler(50*time.Millisecond, task)

		go scheduler.Run(ctx)
		cancel()

		// then
		assert.Equal(t, 0, task.callCount, "If stopped immediately, Do() should not be called")
	})
}
