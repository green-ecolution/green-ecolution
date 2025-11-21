package subscriber

import (
	"context"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	svcMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestUpdateTreeSubsciber(t *testing.T) {
	t.Run("should handle update event", func(t *testing.T) {
		// given
		tcSvc := svcMock.NewMockTreeClusterService(t)
		sub := NewUpdateTreeSubscriber(tcSvc)
		event := entities.NewEventUpdateTree(nil, nil, nil)

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
		event := entities.NewEventCreateTree(nil, nil)

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
		event := entities.NewEventDeleteTree(nil)

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
		event := entities.NewEventSensorData(nil)

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
		event := entities.NewEventUpdateWateringPlan(nil, nil)

		tcSvc.EXPECT().HandleUpdateWateringPlan(mock.Anything, &event).Return(nil)

		assert.NotPanics(t, func() {
			// when
			err := sub.HandleEvent(context.Background(), event)

			// then
			assert.NoError(t, err)
		})
	})
}
