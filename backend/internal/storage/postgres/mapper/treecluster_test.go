package mapper_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper/generated"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
)

func TestTreeclusterMapper_FromSql(t *testing.T) {
	treeclusterMapper := &generated.InternalTreeClusterRepoMapperImpl{}

	t.Run("should convert from sql to entity", func(t *testing.T) {
		// given
		src := allTestTreecluster[0]

		// when
		got, err := treeclusterMapper.FromSql(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Equal(t, src.ID, got.ID)
		assert.Equal(t, src.CreatedAt, got.CreatedAt)
		assert.Equal(t, src.UpdatedAt, got.UpdatedAt)
		assert.Equal(t, src.WateringStatus, sqlc.WateringStatus(got.WateringStatus))
		assert.Equal(t, src.LastWatered, got.LastWatered)
		assert.Equal(t, src.MoistureLevel, got.MoistureLevel)
		assert.Equal(t, src.Address, got.Address)
		assert.Equal(t, src.Archived, got.Archived)
		assert.Equal(t, src.Latitude, got.Latitude)
		assert.Equal(t, src.Longitude, got.Longitude)
		assert.Equal(t, src.Name, got.Name)
		assert.Equal(t, src.Description, got.Description)
		assert.Equal(t, src.SoilCondition, sqlc.TreeSoilCondition(got.SoilCondition))
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src *sqlc.TreeCluster = nil

		// when
		got, err := treeclusterMapper.FromSql(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

func TestTreeclusterMapper_FromSqlList(t *testing.T) {
	treeclusterMapper := &generated.InternalTreeClusterRepoMapperImpl{}

	t.Run("should convert from sql slice to entity slice", func(t *testing.T) {
		// given
		src := allTestTreecluster

		// when
		got, err := treeclusterMapper.FromSqlList(src)

		// then
		assert.NotNil(t, got)
		assert.NoError(t, err)
		assert.Len(t, got, 2)

		for i, src := range src {
			assert.Equal(t, src.ID, got[i].ID)
			assert.Equal(t, src.CreatedAt, got[i].CreatedAt)
			assert.Equal(t, src.UpdatedAt, got[i].UpdatedAt)
			assert.Equal(t, src.WateringStatus, sqlc.WateringStatus(got[i].WateringStatus))
			assert.Equal(t, src.LastWatered, got[i].LastWatered)
			assert.Equal(t, src.MoistureLevel, got[i].MoistureLevel)
			assert.Equal(t, src.Address, got[i].Address)
			assert.Equal(t, src.Archived, got[i].Archived)
			assert.Equal(t, src.Latitude, got[i].Latitude)
			assert.Equal(t, src.Longitude, got[i].Longitude)
			assert.Equal(t, src.Name, got[i].Name)
			assert.Equal(t, src.Description, got[i].Description)
			assert.Equal(t, src.SoilCondition, sqlc.TreeSoilCondition(got[i].SoilCondition))
		}
	})

	t.Run("should return nil for nil input", func(t *testing.T) {
		// given
		var src []*sqlc.TreeCluster = nil

		// when
		got, err := treeclusterMapper.FromSqlList(src)

		// then
		assert.Nil(t, got)
		assert.NoError(t, err)
	})
}

var now = time.Now()

var allTestTreecluster = []*sqlc.TreeCluster{
	{
		ID:             1,
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
		WateringStatus: sqlc.WateringStatusGood,
		LastWatered:    &now,
		MoistureLevel:  4.10,
		Address:        "123 Garden Lane",
		Description:    "Cluster with newly planted trees",
		Archived:       false,
		SoilCondition:  sqlc.TreeSoilConditionSandig,
		Latitude:       utils.P(41.1234),
		Longitude:      utils.P(-73.9876),
		Name:           "Treecluster 01",
	},
	{
		ID:             2,
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
		WateringStatus: sqlc.WateringStatusGood,
		LastWatered:    &now,
		MoistureLevel:  4.10,
		Address:        "345 Garden Lane",
		Description:    "Cluster needs a lot of care",
		Archived:       false,
		SoilCondition:  sqlc.TreeSoilConditionTonig,
		Latitude:       utils.P(41.1234),
		Longitude:      utils.P(-73.9876),
		Name:           "Treecluster 02",
	},
}

func TestMapWateringStatus(t *testing.T) {
	tests := []struct {
		input    sqlc.WateringStatus
		expected entities.WateringStatus
	}{
		{input: sqlc.WateringStatusGood, expected: entities.WateringStatusGood},
		{input: sqlc.WateringStatusModerate, expected: entities.WateringStatusModerate},
		{input: sqlc.WateringStatusBad, expected: entities.WateringStatusBad},
		{input: sqlc.WateringStatusUnknown, expected: entities.WateringStatusUnknown},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("should return %v for input %v", test.expected, test.input), func(t *testing.T) {
			result := mapper.MapWateringStatus(test.input)
			assert.Equal(t, test.expected, result)
		})
	}
}

func TestMapSoilCondition(t *testing.T) {
	tests := []struct {
		input    sqlc.TreeSoilCondition
		expected entities.TreeSoilCondition
	}{
		{input: sqlc.TreeSoilConditionSandig, expected: entities.TreeSoilConditionSandig},
		{input: sqlc.TreeSoilConditionTonig, expected: entities.TreeSoilConditionTonig},
		{input: sqlc.TreeSoilConditionLehmig, expected: entities.TreeSoilConditionLehmig},
		{input: sqlc.TreeSoilConditionSchluffig, expected: entities.TreeSoilConditionSchluffig},
		{input: sqlc.TreeSoilConditionUnknown, expected: entities.TreeSoilConditionUnknown},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("should return %v for input %v", test.expected, test.input), func(t *testing.T) {
			result := mapper.MapSoilCondition(test.input)
			assert.Equal(t, test.expected, result)
		})
	}
}
