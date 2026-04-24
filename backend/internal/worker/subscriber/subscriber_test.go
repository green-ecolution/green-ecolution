package subscriber

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	svcMock "github.com/green-ecolution/green-ecolution/backend/internal/application/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
)

func TestUpdateTreeSubsciber(t *testing.T) {
	t.Run("should handle update event", func(t *testing.T) {
		// given
		tcSvc := svcMock.NewMockTreeClusterService(t)
		sub := NewUpdateTreeSubscriber(tcSvc)
		event := tree.NewEventUpdate(nil, nil, nil)

		tcSvc.EXPECT().HandleUpdateTree(mock.Anything, &event).Return(nil)

		assert.NotPanics(t, func() {
			// when
			err := sub.HandleEvent(context.Background(), event)

			// then
			assert.NoError(t, err)
		})
	})

	t.Run("should handle create event", func(t *testing.T) {
		// given
		tcSvc := svcMock.NewMockTreeClusterService(t)
		sub := NewCreateTreeSubscriber(tcSvc)
		event := tree.NewEventCreate(nil, nil)

		tcSvc.EXPECT().HandleCreateTree(mock.Anything, &event).Return(nil)

		assert.NotPanics(t, func() {
			// when
			err := sub.HandleEvent(context.Background(), event)

			// then
			assert.NoError(t, err)
		})
	})

	t.Run("should handle delete event", func(t *testing.T) {
		// given
		tcSvc := svcMock.NewMockTreeClusterService(t)
		sub := NewDeleteTreeSubscriber(tcSvc)
		event := tree.NewEventDelete(nil)

		tcSvc.EXPECT().HandleDeleteTree(mock.Anything, &event).Return(nil)

		assert.NotPanics(t, func() {
			// when
			err := sub.HandleEvent(context.Background(), event)

			// then
			assert.NoError(t, err)
		})
	})

	t.Run("should handle new sensor data event", func(t *testing.T) {
		// given
		tcSvc := svcMock.NewMockTreeClusterService(t)
		tSvc := svcMock.NewMockTreeService(t)
		sub := NewSensorDataSubscriber(tcSvc, tSvc)
		event := sensor.NewEventNewData(nil)

		tSvc.EXPECT().HandleNewSensorData(mock.Anything, &event).Return(nil)
		tcSvc.EXPECT().HandleNewSensorData(mock.Anything, &event).Return(nil)

		assert.NotPanics(t, func() {
			// when
			err := sub.HandleEvent(context.Background(), event)

			// then
			assert.NoError(t, err)
		})
	})

	t.Run("should handle update watering plan event", func(t *testing.T) {
		// given
		tcSvc := svcMock.NewMockTreeClusterService(t)
		sub := NewUpdateWateringPlanSubscriber(tcSvc)
		event := watering.NewEventUpdate(nil, nil)

		tcSvc.EXPECT().HandleUpdateWateringPlan(mock.Anything, &event).Return(nil)

		assert.NotPanics(t, func() {
			// when
			err := sub.HandleEvent(context.Background(), event)

			// then
			assert.NoError(t, err)
		})
	})
}
