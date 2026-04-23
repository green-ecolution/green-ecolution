package routing

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func TestDummyRoutingRepo_GenerateRoute(t *testing.T) {
	t.Run("should return ErrRoutingServiceDisabled", func(t *testing.T) {
		// given
		repo := NewDummyRoutingRepo()

		// when
		result, err := repo.GenerateRoute(context.Background(), nil, nil)

		// then
		assert.Nil(t, result)
		assert.ErrorIs(t, err, entities.ErrRoutingServiceDisabled)
	})
}

func TestDummyRoutingRepo_GenerateRawGpxRoute(t *testing.T) {
	t.Run("should return ErrRoutingServiceDisabled", func(t *testing.T) {
		// given
		repo := NewDummyRoutingRepo()

		// when
		result, err := repo.GenerateRawGpxRoute(context.Background(), nil, nil)

		// then
		assert.Nil(t, result)
		assert.ErrorIs(t, err, entities.ErrRoutingServiceDisabled)
	})
}

func TestDummyRoutingRepo_GenerateRouteInformation(t *testing.T) {
	t.Run("should return ErrRoutingServiceDisabled", func(t *testing.T) {
		// given
		repo := NewDummyRoutingRepo()

		// when
		result, err := repo.GenerateRouteInformation(context.Background(), nil, nil)

		// then
		assert.Nil(t, result)
		assert.ErrorIs(t, err, entities.ErrRoutingServiceDisabled)
	})
}
