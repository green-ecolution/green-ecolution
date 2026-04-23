package subscriber

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	svcMock "github.com/green-ecolution/green-ecolution/backend/internal/application/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func TestUpdateTreeSubsciber(t *testing.T) {
	t.Run("should handle update event", func(t *testing.T) {
		// given
		tcSvc := svcMock.NewMockTreeClusterService(t)
		sub := NewUpdateTreeSubscriber(tcSvc)
		event := shared.NewEventUpdateTree(nil, nil, nil)

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
		event := shared.NewEventCreateTree(nil, nil)

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
		event := shared.NewEventDeleteTree(nil)

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
		event := shared.NewEventSensorData(nil)

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
		event := shared.NewEventUpdateWateringPlan(nil, nil)

		tcSvc.EXPECT().HandleUpdateWateringPlan(mock.Anything, &event).Return(nil)

		assert.NotPanics(t, func() {
			// when
			err := sub.HandleEvent(context.Background(), event)

			// then
			assert.NoError(t, err)
		})
	})
}
