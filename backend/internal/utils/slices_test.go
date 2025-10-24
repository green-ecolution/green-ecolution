package utils

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestFilter(t *testing.T) {
	t.Run("should filter even numbers from slice", func(t *testing.T) {
		slice := []int{1, 2, 3, 4, 5}
		result := Filter(slice, func(i int) bool {
			return i%2 == 0
		})
		assert.Equal(t, []int{2, 4}, result)
	})

	t.Run("should return empty slice when no elements match predicate", func(t *testing.T) {
		slice := []int{1, 3, 5}
		result := Filter(slice, func(i int) bool {
			return i%2 == 0
		})
		assert.Empty(t, result)
	})

	t.Run("should return original slice when all elements match predicate", func(t *testing.T) {
		slice := []int{2, 4, 6}
		result := Filter(slice, func(i int) bool {
			return i%2 == 0
		})
		assert.Equal(t, slice, result)
	})

	t.Run("should handle empty slice", func(t *testing.T) {
		slice := []int{}
		result := Filter(slice, func(i int) bool {
			return i%2 == 0
		})
		assert.Empty(t, result)
	})

	t.Run("should filter with complex predicate", func(t *testing.T) {
		slice := []string{"apple", "banana", "cherry", "date"}
		result := Filter(slice, func(s string) bool {
			return len(s) > 5
		})
		assert.Equal(t, []string{"banana", "cherry"}, result)
	})
}

func TestMap(t *testing.T) {
	t.Run("should square all elements in slice", func(t *testing.T) {
		slice := []int{1, 2, 3}
		result := Map(slice, func(i int) int {
			return i * i
		})
		assert.Equal(t, []int{1, 4, 9}, result)
	})

	t.Run("should handle empty slice", func(t *testing.T) {
		slice := []int{}
		result := Map(slice, func(i int) int {
			return i * i
		})
		assert.Empty(t, result)
	})

	t.Run("should map integers to strings", func(t *testing.T) {
		slice := []int{1, 2, 3}
		result := Map(slice, func(i int) string {
			return "num:" + string(rune(i+48))
		})
		assert.Equal(t, []string{"num:1", "num:2", "num:3"}, result)
	})

	t.Run("should double each element in float slice", func(t *testing.T) {
		slice := []float64{1.1, 2.2, 3.3}
		result := Map(slice, func(f float64) float64 {
			return f * 2
		})
		assert.Equal(t, []float64{2.2, 4.4, 6.6}, result)
	})
}

func TestReduce(t *testing.T) {
	t.Run("should sum all elements in slice", func(t *testing.T) {
		slice := []int{1, 2, 3, 4}
		result := Reduce(slice, func(acc, i int) int {
			return acc + i
		}, 0)
		assert.Equal(t, 10, result)
	})

	t.Run("should handle empty slice with initial value", func(t *testing.T) {
		slice := []int{}
		result := Reduce(slice, func(acc, i int) int {
			return acc + i
		}, 10)
		assert.Equal(t, 10, result)
	})

	t.Run("should multiply all elements in slice", func(t *testing.T) {
		slice := []int{1, 2, 3}
		result := Reduce(slice, func(acc, i int) int {
			return acc * i
		}, 1)
		assert.Equal(t, 6, result)
	})

	t.Run("should concatenate strings in slice", func(t *testing.T) {
		slice := []string{"hello", " ", "world"}
		result := Reduce(slice, func(acc, s string) string {
			return acc + s
		}, "")
		assert.Equal(t, "hello world", result)
	})

	t.Run("should find max value in slice", func(t *testing.T) {
		slice := []int{3, 5, 2, 8, 6}
		result := Reduce(slice, func(max, i int) int {
			if i > max {
				return i
			}
			return max
		}, slice[0])
		assert.Equal(t, 8, result)
	})
}

func TestContains(t *testing.T) {
	t.Run("should return true if element is found", func(t *testing.T) {
		slice := []int{1, 2, 3, 4, 5}
		result := Contains(slice, 3)
		assert.True(t, result)
	})

	t.Run("should return false if element is not found", func(t *testing.T) {
		slice := []int{1, 2, 4, 5}
		result := Contains(slice, 3)
		assert.False(t, result)
	})

	t.Run("should handle empty slice", func(t *testing.T) {
		slice := []int{}
		result := Contains(slice, 1)
		assert.False(t, result)
	})

	t.Run("should handle string slice", func(t *testing.T) {
		slice := []string{"apple", "banana", "cherry"}
		result := Contains(slice, "banana")
		assert.True(t, result)

		result = Contains(slice, "date")
		assert.False(t, result)
	})

	t.Run("should handle slice with boolean values", func(t *testing.T) {
		slice := []bool{true, false, true}
		result := Contains(slice, false)
		assert.True(t, result)

		result = Contains(slice, true)
		assert.True(t, result)
	})
}
