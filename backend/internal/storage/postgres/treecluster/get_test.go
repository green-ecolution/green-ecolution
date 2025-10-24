package treecluster

import (
	"context"
	"fmt"
	"sort"
	"testing"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTreeClusterRepository_GetAll(t *testing.T) {
	t.Run("should return all tree clusters ordered by name without limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		got, totalCount, err := r.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Len(t, got, len(allTestCluster))
		assert.Equal(t, totalCount, int64(len(allTestCluster)))

		sortedTestCluster := sortClusterByName(allTestCluster)

		for i, tc := range got {
			assert.Equal(t, sortedTestCluster[i].ID, tc.ID)
			assert.Equal(t, sortedTestCluster[i].Name, tc.Name)

			// assert region
			if sortedTestCluster[i].RegionID == -1 {
				assert.Nil(t, tc.Region)
				assert.NoError(t, err)
			} else {
				assert.NotNil(t, tc.Region)
				assert.Equal(t, sortedTestCluster[i].RegionID, tc.Region.ID)
			}

			// assert trees
			assert.Len(t, tc.Trees, len(sortedTestCluster[i].TreeIDs))
			if len(sortedTestCluster[i].TreeIDs) == 0 {
				assert.Empty(t, tc.Trees)
			}

			for j, tree := range tc.Trees {
				assert.NotZero(t, tree)
				assert.Equal(t, sortedTestCluster[i].TreeIDs[j], tree.ID)
			}
		}
	})

	t.Run("should return all tree clusters with provider", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		expectedCluster := allTestCluster[len(allTestCluster)-1]

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		got, totalCount, err := r.GetAll(ctx, entities.TreeClusterQuery{Query: entities.Query{Provider: "test-provider"}})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, totalCount, int64(1))
		assert.Equal(t, expectedCluster.ID, got[0].ID)
		assert.Equal(t, expectedCluster.Name, got[0].Name)
		assert.Equal(t, expectedCluster.Provider, got[0].Provider)
		assert.Equal(t, expectedCluster.AdditionalInfo, got[0].AdditionalInfo)
	})

	t.Run("should return tree clusters ordered by name limited by 2 and with an offset of 2", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Len(t, got, 2)
		assert.Equal(t, totalCount, int64(len(allTestCluster)))

		sortedTestCluster := sortClusterByName(allTestCluster)
		sortedTestCluster = sortedTestCluster[2:4]

		for i, tc := range got {
			assert.Equal(t, sortedTestCluster[i].ID, tc.ID)
			assert.Equal(t, sortedTestCluster[i].Name, tc.Name)
		}
	})

	t.Run("should return error on invalid page value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(0))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error on invalid limit value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(0))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return empty slice when db is empty", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		_, _, err := r.GetAll(ctx, entities.TreeClusterQuery{})

		// then
		assert.Error(t, err)
	})

	t.Run("should return tree clusters filtered by watering status", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := entities.TreeClusterQuery{
			WateringStatuses: []entities.WateringStatus{entities.WateringStatusGood},
		}

		// when
		got, totalCount, err := r.GetAll(ctx, filter)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, int64(len(got)), totalCount)

		for _, cluster := range got {
			assert.Equal(t, entities.WateringStatusGood, cluster.WateringStatus)
		}
	})

	t.Run("should return tree clusters filtered by region", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := entities.TreeClusterQuery{
			Regions: []string{"Mürwik"},
		}

		// when
		got, totalCount, err := r.GetAll(ctx, filter)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, int64(len(got)), totalCount)

		for _, cluster := range got {
			assert.NotNil(t, cluster.Region)
			assert.Equal(t, "Mürwik", cluster.Region.Name)
		}
	})

	t.Run("should return tree clusters filtered by both watering status and region", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := entities.TreeClusterQuery{
			WateringStatuses: []entities.WateringStatus{entities.WateringStatusModerate},
			Regions:          []string{"Mürwik"},
		}

		// when
		got, totalCount, err := r.GetAll(ctx, filter)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, int64(len(got)), totalCount)

		for _, cluster := range got {
			assert.Equal(t, entities.WateringStatusModerate, cluster.WateringStatus)
			assert.NotNil(t, cluster.Region)
			assert.Equal(t, "Mürwik", cluster.Region.Name)
		}
	})

	t.Run("should return tree clusters filtered by multiple watering statuses and multiple regions", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		wateringstatues := []entities.WateringStatus{
			entities.WateringStatusGood,
			entities.WateringStatusModerate,
		}
		regionNames := []string{"Mürwik", "Altstadt"}

		filter := entities.TreeClusterQuery{
			WateringStatuses: wateringstatues,
			Regions:          regionNames,
			Query:            entities.Query{Provider: ""},
		}

		// when
		got, totalCount, err := r.GetAll(ctx, filter)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, int64(len(got)), totalCount)

		for _, cluster := range got {
			assert.Contains(t,
				wateringstatues, cluster.WateringStatus, "Cluster has a status outside the expected list",
			)

			require.NotNil(t, cluster.Region)
			assert.Contains(t, regionNames, cluster.Region.Name, "Cluster has a region outside the expected list")
		}
	})

	t.Run("should return tree clusters filtered by multiple statuses and regions limited by 2 and with an offset of 1", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(2))

		wateringstatues := []entities.WateringStatus{
			entities.WateringStatusGood,
			entities.WateringStatusModerate,
		}
		regionNames := []string{"Mürwik", "Altstadt"}

		filter := entities.TreeClusterQuery{
			WateringStatuses: wateringstatues,
			Regions:          regionNames,
			Query:            entities.Query{Provider: ""},
		}

		// when
		got, totalCount, err := r.GetAll(ctx, filter)

		fmt.Println(got)
		fmt.Println(totalCount)
		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, int64(len(got)), totalCount)

		for _, cluster := range got {
			assert.Contains(t,
				wateringstatues, cluster.WateringStatus, "Cluster has a status outside the expected list",
			)

			require.NotNil(t, cluster.Region)
			assert.Contains(t, regionNames, cluster.Region.Name, "Cluster has a region outside the expected list")
		}
	})

	t.Run("should return empty list if multiple statuses and regions do not match any cluster", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := entities.TreeClusterQuery{
			WateringStatuses: []entities.WateringStatus{
				entities.WateringStatusBad,
				entities.WateringStatusUnknown,
			},
			Regions: []string{"DoesNotExist", "FarAwayLand"},
			Query:   entities.Query{Provider: ""},
		}

		// when
		got, totalCount, err := r.GetAll(ctx, filter)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
		assert.Equal(t, int64(0), totalCount)
	})
}

func TestTreeClusterRepository_GetCount(t *testing.T) {
	t.Run("should return count of all tree cluster in db", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		// when
		totalCount, err := r.GetCount(context.Background(), entities.TreeClusterQuery{})

		// then
		assert.NoError(t, err)
		assert.Equal(t, int64(len(allTestCluster)), totalCount)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		totalCount, err := r.GetCount(ctx, entities.TreeClusterQuery{})

		// then
		assert.Error(t, err)
		assert.Equal(t, int64(0), totalCount)
	})
}

func TestTreeClusterRepository_GetByID(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
	t.Run("should return tree cluster by id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 1)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, allTestCluster[0].ID, got.ID)
		assert.Equal(t, allTestCluster[0].Name, got.Name)
		assert.Equal(t, allTestCluster[0].Address, got.Address)
		assert.Equal(t, allTestCluster[0].Description, got.Description)
		assert.Equal(t, allTestCluster[0].MoistureLevel, got.MoistureLevel)
		assert.Equal(t, allTestCluster[0].WateringStatus, got.WateringStatus)
		assert.Equal(t, allTestCluster[0].SoilCondition, got.SoilCondition)

		if got.Latitude != nil {
			assert.Equal(t, allTestCluster[0].Latitude, *got.Latitude)
		} else {
			assert.Nil(t, got.Latitude)
		}

		if got.Longitude != nil {
			assert.Equal(t, allTestCluster[0].Longitude, *got.Longitude)
		} else {
			assert.Nil(t, got.Longitude)
		}

		// assert region
		if allTestCluster[0].RegionID == -1 {
			assert.Nil(t, got.Region)
			assert.NoError(t, err)
		} else {
			assert.NotNil(t, got.Region)
			assert.Equal(t, allTestCluster[0].RegionID, got.Region.ID)
		}

		// assert trees
		assert.Len(t, got.Trees, len(allTestCluster[0].TreeIDs))
		if len(allTestCluster[0].TreeIDs) == 0 {
			assert.Empty(t, got.Trees)
		}

		for j, tree := range got.Trees {
			assert.NotZero(t, tree)
			assert.Equal(t, allTestCluster[0].TreeIDs[j], tree.ID)
		}
	})

	t.Run("should return error when tree cluster with non-existing id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 99)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when tree cluster with negative id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), -1)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when tree cluster with zero id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)

		// when
		got, err := r.GetByID(context.Background(), 0)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		got, err := r.GetByID(ctx, 1)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestTreeClusterRepository_GetByIDs(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")

	t.Run("should return tree clusters by ids", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ids := []int32{1, 2}

		// when
		got, err := r.GetByIDs(context.Background(), ids)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Len(t, got, 2)

		for i, cluster := range got {
			assert.Equal(t, allTestCluster[i].ID, cluster.ID)
			assert.Equal(t, allTestCluster[i].Name, cluster.Name)
			assert.Equal(t, allTestCluster[i].Address, cluster.Address)
			assert.Equal(t, allTestCluster[i].MoistureLevel, cluster.MoistureLevel)
			assert.Equal(t, allTestCluster[i].WateringStatus, cluster.WateringStatus)
			assert.Equal(t, allTestCluster[i].SoilCondition, cluster.SoilCondition)
			assert.Equal(t, allTestCluster[i].Description, cluster.Description)

			if cluster.Latitude != nil {
				assert.Equal(t, allTestCluster[i].Latitude, *cluster.Latitude)
			} else {
				assert.Nil(t, cluster.Latitude)
			}

			if cluster.Longitude != nil {
				assert.Equal(t, allTestCluster[i].Longitude, *cluster.Longitude)
			} else {
				assert.Nil(t, cluster.Longitude)
			}

			// assert region
			if allTestCluster[i].RegionID == -1 {
				assert.Nil(t, cluster.Region)
				assert.NoError(t, err)
			} else {
				assert.NotNil(t, cluster.Region)
				assert.Equal(t, allTestCluster[i].RegionID, cluster.Region.ID)
			}

			// assert trees
			assert.Len(t, cluster.Trees, len(allTestCluster[i].TreeIDs))
			if len(allTestCluster[i].TreeIDs) == 0 {
				assert.Empty(t, cluster.Trees)
			}

			for j, tree := range cluster.Trees {
				assert.NotZero(t, tree)
				assert.Equal(t, allTestCluster[i].TreeIDs[j], tree.ID)
			}
		}
	})

	t.Run("should return empty list if no trees are found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeClusterRepository(suite.Store, mappers)
		ids := []int32{99, 100, -1, 0}

		// when
		got, err := r.GetByIDs(context.Background(), ids)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		trees, err := r.GetByIDs(ctx, []int32{1, 2})

		// then
		assert.Error(t, err)
		assert.Nil(t, trees)
	})
}

func TestTreeClusterRepository_GetAllLatestSensorDataByClusterID(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")

	t.Run("shold return all latest sensor data by cluster id", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		tcID := int32(50)

		// when
		got, err := r.GetAllLatestSensorDataByClusterID(context.Background(), tcID)

		// then
		assert.NoError(t, err)
		assert.Len(t, got, 2)
		assert.NotEqual(t, 34.0, got[0].Data.Battery) // based on seed
		assert.Equal(t, 99.0, got[0].Data.Battery)
		assert.NotEqual(t, 34.0, got[1].Data.Battery) // based on seed
		assert.Equal(t, 99.0, got[1].Data.Battery)
	})

	t.Run("shold return empty array when tree cluster not exists", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		tcID := int32(99)

		// when
		got, err := r.GetAllLatestSensorDataByClusterID(context.Background(), tcID)

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
	})
}

func TestVehicleRepository_GetAllWithWateringPlanCount(t *testing.T) {
	t.Run("should return all regions with the associated watering plan count", func(t *testing.T) {
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/storage/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		exptectedRegions := getRegionCounts()

		got, err := r.GetAllRegionsWithWateringPlanCount(context.Background())

		assert.NoError(t, err)
		assert.Equal(t, len(exptectedRegions), len(got))

		for i, entry := range got {
			assert.Equal(t, exptectedRegions[i].Name, entry.Name)
			assert.Equal(t, exptectedRegions[i].WateringPlanCount, entry.WateringPlanCount)
		}
	})

	t.Run("should return empty slice on empty db", func(t *testing.T) {
		suite.ResetDB(t)
		r := NewTreeClusterRepository(suite.Store, mappers)

		got, err := r.GetAllRegionsWithWateringPlanCount(context.Background())

		assert.NoError(t, err)
		assert.Equal(t, 0, len(got))
		assert.Empty(t, got)
	})
}

type testTreeCluster struct {
	ID             int32
	Name           string
	Address        string
	Description    string
	MoistureLevel  float64
	WateringStatus entities.WateringStatus
	Latitude       float64
	Longitude      float64
	SoilCondition  entities.TreeSoilCondition
	RegionID       int32
	TreeIDs        []int32
	Provider       string
	AdditionalInfo map[string]interface{}
}

var allTestCluster = []*testTreeCluster{
	{
		ID:             1,
		Name:           "Solitüde Strand",
		Address:        "Solitüde Strand",
		Description:    "Alle Bäume am Strand",
		MoistureLevel:  0.75,
		WateringStatus: entities.WateringStatusGood,
		Latitude:       54.82094,
		Longitude:      9.489022,
		SoilCondition:  entities.TreeSoilConditionSandig,
		RegionID:       1,
		TreeIDs:        []int32{1, 2, 3},
	},
	{
		ID:             2,
		Name:           "Sankt-Jürgen-Platz",
		Address:        "Ulmenstraße",
		Description:    "Bäume beim Sankt-Jürgen-Platz",
		MoistureLevel:  0.5,
		WateringStatus: entities.WateringStatusModerate,
		Latitude:       54.78805731048199,
		Longitude:      9.44400186680097,
		SoilCondition:  entities.TreeSoilConditionSchluffig,
		RegionID:       1,
		TreeIDs:        []int32{4, 5, 6},
	},
	{
		ID:             3,
		Name:           "Flensburger Stadion",
		Address:        "Flensburger Stadion",
		Description:    "Alle Bäume in der Gegend des Stadions in Mürwik",
		MoistureLevel:  0.7,
		WateringStatus: entities.WateringStatusUnknown,
		Latitude:       54.802163,
		Longitude:      9.446398,
		SoilCondition:  entities.TreeSoilConditionSchluffig,
		RegionID:       1,
		TreeIDs:        []int32{16, 17, 18, 19, 20},
	},
	{
		ID:             4,
		Name:           "Campus Hochschule",
		Address:        "Thomas-Finke Straße",
		Description:    "Gruppe ist besonders anfällig",
		MoistureLevel:  0.1,
		WateringStatus: entities.WateringStatusGood,
		Latitude:       54.77578311851497,
		Longitude:      9.450294506300525,
		SoilCondition:  entities.TreeSoilConditionSchluffig,
		RegionID:       4,
		TreeIDs:        []int32{12, 13, 14, 15},
	},
	{
		ID:             5,
		Name:           "Mathildenstraße",
		Address:        "Mathildenstraße",
		Description:    "Sehr enge Straße und dadurch schlecht zu bewässern.",
		MoistureLevel:  0.4,
		WateringStatus: entities.WateringStatusBad,
		Latitude:       54.78219253876479,
		Longitude:      9.423978982828825,
		SoilCondition:  entities.TreeSoilConditionSchluffig,
		RegionID:       10,
		TreeIDs:        []int32{7, 8, 9, 10, 11},
	},
	{
		ID:             6,
		Name:           "Nordstadt",
		Address:        "Apenrader Straße",
		Description:    "Guter Baumbestand mit großen Kronen.",
		MoistureLevel:  0.6,
		WateringStatus: entities.WateringStatusUnknown,
		Latitude:       54.807162,
		Longitude:      9.423138,
		SoilCondition:  entities.TreeSoilConditionSandig,
		RegionID:       13,
		TreeIDs:        []int32{21, 22, 23, 24},
	},
	{
		ID:             7,
		Name:           "TSB Neustadt",
		Address:        "Ecknerstraße",
		Description:    "Kleiner Baumbestand.",
		MoistureLevel:  0.75,
		WateringStatus: entities.WateringStatusGood,
		Latitude:       54.797162,
		Longitude:      9.41962,
		SoilCondition:  entities.TreeSoilConditionSandig,
		RegionID:       13,
	},
	{
		ID:             8,
		Name:           "Gewerbegebiet Süd",
		Address:        "Address8",
		Description:    "Description8",
		MoistureLevel:  8.0,
		WateringStatus: entities.WateringStatusGood,
		Latitude:       0xc0012fc3d0,
		Longitude:      0xc0012fc3d8,
		SoilCondition:  entities.TreeSoilConditionLehmig,
		RegionID:       -1, // no region
	},
	{
		ID:             50,
		Name:           "Gewerbegebiet Süd",
		Address:        "Address9",
		Description:    "Description9",
		MoistureLevel:  9.0,
		WateringStatus: entities.WateringStatusGood,
		Latitude:       0xc0011fc3d0,
		Longitude:      0xc0011fc3d8,
		SoilCondition:  entities.TreeSoilConditionLehmig,
		RegionID:       -1, // no region
		TreeIDs:        []int32{25, 26, 27, 28},
		Provider:       "test-provider",
		AdditionalInfo: map[string]interface{}{
			"foo": "bar",
		},
	},
}

var allTestRegions = []*entities.Region{
	{
		ID:   1,
		Name: "Mürwik",
	},
	{
		ID:   2,
		Name: "Fruerlund",
	},
	{
		ID:   4,
		Name: "Sandberg",
	},
	{
		ID:   10,
		Name: "Friesischer Berg",
	},
	{
		ID:   13,
		Name: "Nordstadt",
	},
}

func sortClusterByName(data []*testTreeCluster) []*testTreeCluster {
	sorted := make([]*testTreeCluster, len(data))
	copy(sorted, data)

	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].Name < sorted[j].Name
	})

	return sorted
}

type testWateringPlan struct {
	ID           int32
	Date         time.Time
	TreeClusters []*testTreeCluster
}

var allTestWateringPlans = []*testWateringPlan{
	{
		ID:           1,
		Date:         time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
		TreeClusters: allTestCluster[0:2],
	},
	{
		ID:           2,
		Date:         time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
		TreeClusters: allTestCluster[2:4],
	},
	{
		ID:           3,
		Date:         time.Date(2024, 6, 12, 0, 0, 0, 0, time.UTC),
		TreeClusters: allTestCluster[2:6],
	},
	{
		ID:           4,
		Date:         time.Date(2024, 6, 10, 0, 0, 0, 0, time.UTC),
		TreeClusters: allTestCluster[0:5],
	},
	{
		ID:           5,
		Date:         time.Date(2024, 6, 4, 0, 0, 0, 0, time.UTC),
		TreeClusters: allTestCluster[0:4],
	},
}

func getRegionCounts() []*entities.RegionEvaluation {
	regionCountMap := make(map[int32]map[int32]struct{})

	for _, plan := range allTestWateringPlans {
		regionSet := make(map[int32]struct{})

		for _, cluster := range plan.TreeClusters {
			regionSet[cluster.RegionID] = struct{}{}
		}

		for regionID := range regionSet {
			if regionCountMap[regionID] == nil {
				regionCountMap[regionID] = make(map[int32]struct{})
			}
			regionCountMap[regionID][plan.ID] = struct{}{}
		}
	}

	var regionEvaluations []*entities.RegionEvaluation
	for regionID, planMap := range regionCountMap {
		for _, region := range allTestRegions {
			if region.ID == regionID {
				regionEvaluations = append(regionEvaluations, &entities.RegionEvaluation{
					Name:              region.Name,
					WateringPlanCount: int64(len(planMap)),
				})
				break
			}
		}
	}

	sort.Slice(regionEvaluations, func(i, j int) bool {
		return regionEvaluations[i].WateringPlanCount > regionEvaluations[j].WateringPlanCount
	})

	return regionEvaluations
}
