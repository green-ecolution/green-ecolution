package utils

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestMapSlice(t *testing.T) {
	double := func(n int) int { return n * 2 }

	t.Run("nil input returns nil", func(t *testing.T) {
		result := MapSlice[int, int](nil, double)
		assert.Nil(t, result)
	})

	t.Run("empty slice returns empty slice", func(t *testing.T) {
		result := MapSlice([]int{}, double)
		assert.NotNil(t, result)
		assert.Empty(t, result)
	})

	t.Run("maps elements correctly", func(t *testing.T) {
		result := MapSlice([]int{1, 2, 3}, double)
		assert.Equal(t, []int{2, 4, 6}, result)
	})
}

func TestMapSliceErr(t *testing.T) {
	errBoom := errors.New("boom")

	safeDouble := func(n int) (int, error) {
		if n < 0 {
			return 0, errBoom
		}
		return n * 2, nil
	}

	t.Run("nil input returns nil", func(t *testing.T) {
		result, err := MapSliceErr[int, int](nil, safeDouble)
		assert.NoError(t, err)
		assert.Nil(t, result)
	})

	t.Run("empty slice returns empty slice", func(t *testing.T) {
		result, err := MapSliceErr([]int{}, safeDouble)
		assert.NoError(t, err)
		assert.NotNil(t, result)
		assert.Empty(t, result)
	})

	t.Run("maps elements correctly", func(t *testing.T) {
		result, err := MapSliceErr([]int{1, 2, 3}, safeDouble)
		assert.NoError(t, err)
		assert.Equal(t, []int{2, 4, 6}, result)
	})

	t.Run("propagates error and aborts early", func(t *testing.T) {
		result, err := MapSliceErr([]int{1, -1, 3}, safeDouble)
		assert.ErrorIs(t, err, errBoom)
		assert.Nil(t, result)
	})
}
