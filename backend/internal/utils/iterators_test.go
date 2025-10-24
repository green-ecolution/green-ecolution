package utils

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNumSequence(t *testing.T) {
	t.Run("should generate sequence of numbers", func(t *testing.T) {
		// given
		start := 1
		seq := NumberSequence(start)

		// when
		var result []int
		seq(func(i int) bool {
			result = append(result, i)
			return i < 4
		})

		// then
		expected := []int{1, 2, 3, 4}
		assert.Equal(t, expected, result)
	})

	t.Run("should generate sequence of numbers for loop", func(t *testing.T) {
		// when
		var result []int
		for i := range NumberSequence(1) {
			result = append(result, i)

			if i >= 4 {
				break
			}
		}

		// then
		expected := []int{1, 2, 3, 4}
		assert.Equal(t, expected, result)
	})

	t.Run("should generate sequence of numbers starting from 0", func(t *testing.T) {
		// given
		seq := NumberSequence(0)

		// when
		var result []int
		seq(func(i int) bool {
			result = append(result, i)
			return i < 3
		})

		// then
		expected := []int{0, 1, 2, 3}
		assert.Equal(t, expected, result)
	})

	t.Run("should generate sequence of numbers starting from 0 for loop", func(t *testing.T) {
		// when
		var result []int
		for i := range NumberSequence(0) {
			result = append(result, i)

			if i >= 3 {
				break
			}
		}

		// then
		expected := []int{0, 1, 2, 3}
		assert.Equal(t, expected, result)
	})

	t.Run("should generate sequence of numbers starting from 10", func(t *testing.T) {
		// given
		seq := NumberSequence(10)

		// when
		var result []int
		seq(func(i int) bool {
			result = append(result, i)
			return i < 13
		})

		// then
		expected := []int{10, 11, 12, 13}
		assert.Equal(t, expected, result)
	})

	t.Run("should generate sequence of numbers starting from 10 for loop", func(t *testing.T) {
		// when
		var result []int
		for i := range NumberSequence(10) {
			result = append(result, i)

			if i >= 13 {
				break
			}
		}

		// then
		expected := []int{10, 11, 12, 13}
		assert.Equal(t, expected, result)
	})

	t.Run("should generate sequence of numbers starting from -1", func(t *testing.T) {
		// given
		seq := NumberSequence(-1)

		// when
		var result []int
		seq(func(i int) bool {
			result = append(result, i)
			return i < 2
		})

		// then
		expected := []int{-1, 0, 1, 2}
		assert.Equal(t, expected, result)
	})

	t.Run("should generate sequence of numbers starting from -1 for loop", func(t *testing.T) {
		// when
		var result []int
		for i := range NumberSequence(-1) {
			result = append(result, i)

			if i >= 2 {
				break
			}
		}

		// then
		expected := []int{-1, 0, 1, 2}
		assert.Equal(t, expected, result)
	})
}
