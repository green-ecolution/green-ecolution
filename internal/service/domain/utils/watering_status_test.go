package utils

import (
	"context"
	"testing"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/stretchr/testify/assert"
)

func Test_CheckAndSortWatermarks(t *testing.T) {
	t.Run("should split watermarks sensors in correct objects", func(t *testing.T) {
		// given
		watermarks := []entities.Watermark{
			{Depth: 90}, {Depth: 30}, {Depth: 60},
		}

		// when
		w30, w60, w90, err := CheckAndSortWatermarks(watermarks)

		//then
		assert.NoError(t, err)
		assert.Equal(t, w30.Depth, 30)
		assert.Equal(t, w60.Depth, 60)
		assert.Equal(t, w90.Depth, 90)
	})

	t.Run("should return err on unsupported depth", func(t *testing.T) {
		// given
		watermarks := []entities.Watermark{
			{Depth: 42}, {Depth: 69}, {Depth: 420},
		}

		// when
		_, _, _, err := CheckAndSortWatermarks(watermarks)

		//then
		assert.Error(t, err)
	})

	t.Run("should return err on length not three", func(t *testing.T) {
		// given
		watermarks := []entities.Watermark{
			{Depth: 30},
		}

		// when
		_, _, _, err := CheckAndSortWatermarks(watermarks)

		//then
		assert.Error(t, err)
	})
}

func Test_CalculateWateringStatus(t *testing.T) {
	tests := []struct {
		name  string
		input struct {
			plantingYear int32
			watermarks   []entities.Watermark
		}
		output entities.WateringStatus
	}{
		{
			name: "should return good on first year with all sensor good",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 1),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 12},
					{Depth: 60, Centibar: 9},
					{Depth: 90, Centibar: 24},
				},
			},
			output: entities.WateringStatusGood,
		},
		{
			name: "should return moderat on first year with min one moderat and no bad",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 1),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 12},
					{Depth: 60, Centibar: 32},
					{Depth: 90, Centibar: 24},
				},
			},
			output: entities.WateringStatusModerate,
		},
		{
			name: "should return bad on first year with min one bad",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 1),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 12},
					{Depth: 60, Centibar: 31},
					{Depth: 90, Centibar: 33},
				},
			},
			output: entities.WateringStatusBad,
		},
		{
			name: "should return good on second year with all good",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 2),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 61},
					{Depth: 60, Centibar: 9},
					{Depth: 90, Centibar: 24},
				},
			},
			output: entities.WateringStatusGood,
		},
		{
			name: "should return moderate on second year with min one moderate",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 2),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 80},
					{Depth: 60, Centibar: 9},
					{Depth: 90, Centibar: 24},
				},
			},
			output: entities.WateringStatusModerate,
		},
		{
			name: "should return bad on second year with one bad",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 2),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 81},
					{Depth: 60, Centibar: 31},
					{Depth: 90, Centibar: 31},
				},
			},
			output: entities.WateringStatusBad,
		},
		{
			name: "should return good on third year with all good",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 3),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 1584},
					{Depth: 60, Centibar: 9},
					{Depth: 90, Centibar: 24},
				},
			},
			output: entities.WateringStatusGood,
		},
		{
			name: "should return bad on third year with one bad",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 3),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 1585},
					{Depth: 60, Centibar: 31},
					{Depth: 90, Centibar: 31},
				},
			},
			output: entities.WateringStatusBad,
		},
		{
			name: "should return unknown when planting year is greater then 3",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 4),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 1586},
					{Depth: 60, Centibar: 31},
					{Depth: 90, Centibar: 31},
				},
			},
			output: entities.WateringStatusUnknown,
		},
		{
			name: "should return unknown on malformed watermarks",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year() - 2),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 1586},
					{Depth: 90, Centibar: 31},
				},
			},
			output: entities.WateringStatusUnknown,
		},
		{
			name: "should calculate first year when treeLifetime is 0",
			input: struct {
				plantingYear int32
				watermarks   []entities.Watermark
			}{
				plantingYear: int32(time.Now().Year()),
				watermarks: []entities.Watermark{
					{Depth: 30, Centibar: 33},
					{Depth: 60, Centibar: 9},
					{Depth: 90, Centibar: 24},
				},
			},
			output: entities.WateringStatusBad,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// given
			watermarks := tt.input.watermarks
			plantingYear := tt.input.plantingYear

			// when
			got := CalculateWateringStatus(context.Background(), plantingYear, watermarks)

			// then
			assert.Equal(t, tt.output, got)
		})
	}
}
