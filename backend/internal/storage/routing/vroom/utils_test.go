package vroom

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestReduceSteps(t *testing.T) {
	t.Run("should append element when accumulator is empty", func(t *testing.T) {
		// given
		acc := []*VroomRouteStep{}
		current := VroomRouteStep{Type: "start", Location: []float64{1.0, 2.0}}

		// when
		result := ReduceSteps(acc, current)

		// then
		assert.Len(t, result, 1)
		assert.Equal(t, "start", result[0].Type)
	})

	t.Run("should append non-pickup after start", func(t *testing.T) {
		// given
		acc := []*VroomRouteStep{
			{Type: "start", Location: []float64{1.0, 2.0}},
		}
		current := VroomRouteStep{Type: "delivery", Location: []float64{3.0, 4.0}}

		// when
		result := ReduceSteps(acc, current)

		// then
		assert.Len(t, result, 2)
		assert.Equal(t, "delivery", result[1].Type)
	})

	t.Run("should append pickup after start", func(t *testing.T) {
		// given
		acc := []*VroomRouteStep{
			{Type: "start", Location: []float64{1.0, 2.0}},
		}
		current := VroomRouteStep{Type: "pickup", Location: []float64{3.0, 4.0}}

		// when
		result := ReduceSteps(acc, current)

		// then
		assert.Len(t, result, 2)
		assert.Equal(t, "pickup", result[1].Type)
	})

	t.Run("should merge two consecutive pickups", func(t *testing.T) {
		// given
		acc := []*VroomRouteStep{
			{Type: "start", Location: []float64{1.0, 2.0}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{100}},
		}
		current := VroomRouteStep{Type: "pickup", Location: []float64{5.0, 6.0}, Load: []int32{200}}

		// when
		result := ReduceSteps(acc, current)

		// then
		assert.Len(t, result, 2)
		assert.Equal(t, "pickup", result[1].Type)
		assert.Equal(t, []int32{200}, result[1].Load)
	})

	t.Run("should append delivery after pickup", func(t *testing.T) {
		// given
		acc := []*VroomRouteStep{
			{Type: "start", Location: []float64{1.0, 2.0}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{100}},
		}
		current := VroomRouteStep{Type: "delivery", Location: []float64{5.0, 6.0}}

		// when
		result := ReduceSteps(acc, current)

		// then
		assert.Len(t, result, 3)
		assert.Equal(t, "delivery", result[2].Type)
	})

	t.Run("should reduce full sequence with consecutive pickups", func(t *testing.T) {
		// given
		steps := []VroomRouteStep{
			{Type: "start", Location: []float64{1.0, 2.0}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{100}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{200}},
			{Type: "delivery", Location: []float64{5.0, 6.0}, Load: []int32{50}},
		}

		// when
		acc := make([]*VroomRouteStep, 0)
		for _, step := range steps {
			acc = ReduceSteps(acc, step)
		}

		// then
		assert.Len(t, acc, 3)
		assert.Equal(t, "start", acc[0].Type)
		assert.Equal(t, "pickup", acc[1].Type)
		assert.Equal(t, []int32{200}, acc[1].Load)
		assert.Equal(t, "delivery", acc[2].Type)
	})

	t.Run("should reduce multiple pickup-delivery groups independently", func(t *testing.T) {
		// given
		steps := []VroomRouteStep{
			{Type: "start", Location: []float64{1.0, 2.0}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{100}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{200}},
			{Type: "delivery", Location: []float64{5.0, 6.0}, Load: []int32{50}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{300}},
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{400}},
			{Type: "delivery", Location: []float64{7.0, 8.0}, Load: []int32{25}},
		}

		// when
		acc := make([]*VroomRouteStep, 0)
		for _, step := range steps {
			acc = ReduceSteps(acc, step)
		}

		// then
		assert.Len(t, acc, 5)
		assert.Equal(t, "start", acc[0].Type)
		assert.Equal(t, "pickup", acc[1].Type)
		assert.Equal(t, []int32{200}, acc[1].Load)
		assert.Equal(t, "delivery", acc[2].Type)
		assert.Equal(t, "pickup", acc[3].Type)
		assert.Equal(t, []int32{400}, acc[3].Load)
		assert.Equal(t, "delivery", acc[4].Type)
	})

	t.Run("should not change sequence without pickups", func(t *testing.T) {
		// given
		steps := []VroomRouteStep{
			{Type: "start", Location: []float64{1.0, 2.0}},
			{Type: "delivery", Location: []float64{3.0, 4.0}},
			{Type: "delivery", Location: []float64{5.0, 6.0}},
			{Type: "end", Location: []float64{7.0, 8.0}},
		}

		// when
		acc := make([]*VroomRouteStep, 0)
		for _, step := range steps {
			acc = ReduceSteps(acc, step)
		}

		// then
		assert.Len(t, acc, 4)
	})

	t.Run("should update load field correctly on merge", func(t *testing.T) {
		// given
		acc := []*VroomRouteStep{
			{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{100, 50}},
		}
		current := VroomRouteStep{Type: "pickup", Location: []float64{3.0, 4.0}, Load: []int32{200, 75}}

		// when
		result := ReduceSteps(acc, current)

		// then
		assert.Len(t, result, 1)
		assert.Equal(t, []int32{200, 75}, result[0].Load)
	})
}

func TestRefillCount(t *testing.T) {
	t.Run("should return 0 for empty steps", func(t *testing.T) {
		// given
		steps := []*VroomRouteStep{}

		// when
		result := RefillCount(steps)

		// then
		assert.Equal(t, 0, result)
	})

	t.Run("should return 0 when no pickups", func(t *testing.T) {
		// given
		steps := []*VroomRouteStep{
			{Type: "start"},
			{Type: "delivery"},
			{Type: "end"},
		}

		// when
		result := RefillCount(steps)

		// then
		assert.Equal(t, 0, result)
	})

	t.Run("should return 1 for single pickup", func(t *testing.T) {
		// given
		steps := []*VroomRouteStep{
			{Type: "start"},
			{Type: "pickup"},
			{Type: "delivery"},
		}

		// when
		result := RefillCount(steps)

		// then
		assert.Equal(t, 1, result)
	})

	t.Run("should return correct count for multiple pickups", func(t *testing.T) {
		// given
		steps := []*VroomRouteStep{
			{Type: "start"},
			{Type: "pickup"},
			{Type: "delivery"},
			{Type: "pickup"},
			{Type: "delivery"},
			{Type: "pickup"},
			{Type: "delivery"},
		}

		// when
		result := RefillCount(steps)

		// then
		assert.Equal(t, 3, result)
	})

	t.Run("should count consecutive pickups individually", func(t *testing.T) {
		// given
		steps := []*VroomRouteStep{
			{Type: "start"},
			{Type: "pickup"},
			{Type: "pickup"},
			{Type: "delivery"},
		}

		// when
		result := RefillCount(steps)

		// then
		assert.Equal(t, 2, result)
	})
}
