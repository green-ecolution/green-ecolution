package treecluster

import (
	"context"
	"fmt"
	"sort"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func TestTreeClusterRepository_GetAll(t *testing.T) {
	t.Run("should return all tree clusters ordered by name without limitation", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{})

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
				assert.Nil(t, tc.RegionID)
				assert.NoError(t, err)
			} else {
				assert.NotNil(t, tc.RegionID)
				assert.Equal(t, sortedTestCluster[i].RegionID, *tc.RegionID)
			}

			// assert trees
			assert.Len(t, tc.TreeIDs, len(sortedTestCluster[i].TreeIDs))
			if len(sortedTestCluster[i].TreeIDs) == 0 {
				assert.Empty(t, tc.TreeIDs)
			}

			for j, treeID := range tc.TreeIDs {
				assert.NotZero(t, treeID)
				assert.Equal(t, sortedTestCluster[i].TreeIDs[j], treeID)
			}
		}
	})

	t.Run("should return all tree clusters with provider", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		expectedCluster := allTestCluster[len(allTestCluster)-1]

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{Query: shared.Query{Provider: "test-provider"}})

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
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{})

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
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(0))
		ctx = context.WithValue(ctx, "limit", int32(2))

		// when
		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{})

		// then
		assert.Error(t, err)
		assert.Empty(t, got)
		assert.Equal(t, totalCount, int64(0))
	})

	t.Run("should return error on invalid limit value", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(2))
		ctx = context.WithValue(ctx, "limit", int32(0))

		// when
		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{})

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
		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{})

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
		_, _, err := r.GetAll(ctx, cluster.TreeClusterQuery{})

		// then
		assert.Error(t, err)
	})

	t.Run("should return tree clusters filtered by watering status", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := cluster.TreeClusterQuery{
			WateringStatuses: []shared.WateringStatus{shared.WateringStatusGood},
		}

		// when
		got, totalCount, err := r.GetAll(ctx, filter)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.NotEmpty(t, got)
		assert.Equal(t, int64(len(got)), totalCount)

		for _, cluster := range got {
			assert.Equal(t, shared.WateringStatusGood, cluster.WateringStatus)
		}
	})

	t.Run("should return tree clusters filtered by region", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := cluster.TreeClusterQuery{
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
			assert.NotNil(t, cluster.RegionID)
		}
	})

	t.Run("should return tree clusters filtered by both watering status and region", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := cluster.TreeClusterQuery{
			WateringStatuses: []shared.WateringStatus{shared.WateringStatusModerate},
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
			assert.Equal(t, shared.WateringStatusModerate, cluster.WateringStatus)
			assert.NotNil(t, cluster.RegionID)
		}
	})

	t.Run("should return tree clusters filtered by multiple watering statuses and multiple regions", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		wateringstatues := []shared.WateringStatus{
			shared.WateringStatusGood,
			shared.WateringStatusModerate,
		}
		regionNames := []string{"Mürwik", "Altstadt"}

		filter := cluster.TreeClusterQuery{
			WateringStatuses: wateringstatues,
			Regions:          regionNames,
			Query:            shared.Query{Provider: ""},
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

			require.NotNil(t, cluster.RegionID)
		}
	})

	t.Run("should return tree clusters filtered by multiple statuses and regions limited by 2 and with an offset of 1", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(2))

		wateringstatues := []shared.WateringStatus{
			shared.WateringStatusGood,
			shared.WateringStatusModerate,
		}
		regionNames := []string{"Mürwik", "Altstadt"}

		filter := cluster.TreeClusterQuery{
			WateringStatuses: wateringstatues,
			Regions:          regionNames,
			Query:            shared.Query{Provider: ""},
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

			require.NotNil(t, cluster.RegionID)
		}
	})

	t.Run("should return empty list if multiple statuses and regions do not match any cluster", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		filter := cluster.TreeClusterQuery{
			WateringStatuses: []shared.WateringStatus{
				shared.WateringStatusBad,
				shared.WateringStatusUnknown,
			},
			Regions: []string{"DoesNotExist", "FarAwayLand"},
			Query:   shared.Query{Provider: ""},
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
		suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
		r := NewTreeClusterRepository(suite.Store, mappers)
		// when
		totalCount, err := r.GetCount(context.Background(), cluster.TreeClusterQuery{})

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
		totalCount, err := r.GetCount(ctx, cluster.TreeClusterQuery{})

		// then
		assert.Error(t, err)
		assert.Equal(t, int64(0), totalCount)
	})
}

func TestTreeClusterRepository_GetByID(t *testing.T) {
	suite.ResetDB(t)
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")
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

		if got.Coordinate != nil {
			assert.Equal(t, allTestCluster[0].Latitude, got.Coordinate.Latitude())
			assert.Equal(t, allTestCluster[0].Longitude, got.Coordinate.Longitude())
		} else {
			assert.Nil(t, got.Coordinate)
		}

		// assert region
		if allTestCluster[0].RegionID == -1 {
			assert.Nil(t, got.RegionID)
			assert.NoError(t, err)
		} else {
			assert.NotNil(t, got.RegionID)
			assert.Equal(t, allTestCluster[0].RegionID, *got.RegionID)
		}

		// assert trees
		assert.Len(t, got.TreeIDs, len(allTestCluster[0].TreeIDs))
		if len(allTestCluster[0].TreeIDs) == 0 {
			assert.Empty(t, got.TreeIDs)
		}

		for j, treeID := range got.TreeIDs {
			assert.NotZero(t, treeID)
			assert.Equal(t, allTestCluster[0].TreeIDs[j], treeID)
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
	suite.InsertSeed(t, "internal/infrastructure/postgres/seed/test/treecluster")

	t.Run("should return tree clusters by ids", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ids := []int32{1, 2}

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{IDs: ids})

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, int64(2), totalCount)
		assert.Len(t, got, 2)

		// GetAll sorts by name ASC, so filter and sort expected data accordingly
		var expectedClusters []*testTreeCluster
		for _, tc := range allTestCluster {
			for _, id := range ids {
				if tc.ID == id {
					expectedClusters = append(expectedClusters, tc)
				}
			}
		}
		sortedExpected := sortClusterByName(expectedClusters)

		for i, tc := range got {
			assert.Equal(t, sortedExpected[i].ID, tc.ID)
			assert.Equal(t, sortedExpected[i].Name, tc.Name)
			assert.Equal(t, sortedExpected[i].Address, tc.Address)
			assert.Equal(t, sortedExpected[i].MoistureLevel, tc.MoistureLevel)
			assert.Equal(t, sortedExpected[i].WateringStatus, tc.WateringStatus)
			assert.Equal(t, sortedExpected[i].SoilCondition, tc.SoilCondition)
			assert.Equal(t, sortedExpected[i].Description, tc.Description)

			if tc.Coordinate != nil {
				assert.Equal(t, sortedExpected[i].Latitude, tc.Coordinate.Latitude())
				assert.Equal(t, sortedExpected[i].Longitude, tc.Coordinate.Longitude())
			} else {
				assert.Nil(t, tc.Coordinate)
			}

			// assert region
			if sortedExpected[i].RegionID == -1 {
				assert.Nil(t, tc.RegionID)
				assert.NoError(t, err)
			} else {
				assert.NotNil(t, tc.RegionID)
				assert.Equal(t, sortedExpected[i].RegionID, *tc.RegionID)
			}

			// assert trees
			assert.Len(t, tc.TreeIDs, len(sortedExpected[i].TreeIDs))
			if len(sortedExpected[i].TreeIDs) == 0 {
				assert.Empty(t, tc.TreeIDs)
			}

			for j, treeID := range tc.TreeIDs {
				assert.NotZero(t, treeID)
				assert.Equal(t, sortedExpected[i].TreeIDs[j], treeID)
			}
		}
	})

	t.Run("should return empty list if no clusters are found", func(t *testing.T) {
		// given
		suite.ResetDB(t)
		r := NewTreeClusterRepository(suite.Store, mappers)
		ids := []int32{99, 100}

		ctx := context.WithValue(context.Background(), "page", int32(1))
		ctx = context.WithValue(ctx, "limit", int32(-1))

		// when
		got, totalCount, err := r.GetAll(ctx, cluster.TreeClusterQuery{IDs: ids})

		// then
		assert.NoError(t, err)
		assert.Empty(t, got)
		assert.Equal(t, int64(0), totalCount)
	})

	t.Run("should return error when context is canceled", func(t *testing.T) {
		// given
		r := NewTreeClusterRepository(suite.Store, mappers)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		_, _, err := r.GetAll(ctx, cluster.TreeClusterQuery{IDs: []int32{1, 2}})

		// then
		assert.Error(t, err)
	})
}

type testTreeCluster struct {
	ID             int32
	Name           string
	Address        string
	Description    string
	MoistureLevel  float64
	WateringStatus shared.WateringStatus
	Latitude       float64
	Longitude      float64
	SoilCondition  cluster.TreeSoilCondition
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
		WateringStatus: shared.WateringStatusGood,
		Latitude:       54.82094,
		Longitude:      9.489022,
		SoilCondition:  cluster.TreeSoilConditionSandig,
		RegionID:       1,
		TreeIDs:        []int32{1, 2, 3},
	},
	{
		ID:             2,
		Name:           "Sankt-Jürgen-Platz",
		Address:        "Ulmenstraße",
		Description:    "Bäume beim Sankt-Jürgen-Platz",
		MoistureLevel:  0.5,
		WateringStatus: shared.WateringStatusModerate,
		Latitude:       54.78805731048199,
		Longitude:      9.44400186680097,
		SoilCondition:  cluster.TreeSoilConditionSchluffig,
		RegionID:       1,
		TreeIDs:        []int32{4, 5, 6},
	},
	{
		ID:             3,
		Name:           "Flensburger Stadion",
		Address:        "Flensburger Stadion",
		Description:    "Alle Bäume in der Gegend des Stadions in Mürwik",
		MoistureLevel:  0.7,
		WateringStatus: shared.WateringStatusUnknown,
		Latitude:       54.802163,
		Longitude:      9.446398,
		SoilCondition:  cluster.TreeSoilConditionSchluffig,
		RegionID:       1,
		TreeIDs:        []int32{16, 17, 18, 19, 20},
	},
	{
		ID:             4,
		Name:           "Campus Hochschule",
		Address:        "Thomas-Finke Straße",
		Description:    "Gruppe ist besonders anfällig",
		MoistureLevel:  0.1,
		WateringStatus: shared.WateringStatusGood,
		Latitude:       54.77578311851497,
		Longitude:      9.450294506300525,
		SoilCondition:  cluster.TreeSoilConditionSchluffig,
		RegionID:       4,
		TreeIDs:        []int32{12, 13, 14, 15},
	},
	{
		ID:             5,
		Name:           "Mathildenstraße",
		Address:        "Mathildenstraße",
		Description:    "Sehr enge Straße und dadurch schlecht zu bewässern.",
		MoistureLevel:  0.4,
		WateringStatus: shared.WateringStatusBad,
		Latitude:       54.78219253876479,
		Longitude:      9.423978982828825,
		SoilCondition:  cluster.TreeSoilConditionSchluffig,
		RegionID:       10,
		TreeIDs:        []int32{7, 8, 9, 10, 11},
	},
	{
		ID:             6,
		Name:           "Nordstadt",
		Address:        "Apenrader Straße",
		Description:    "Guter Baumbestand mit großen Kronen.",
		MoistureLevel:  0.6,
		WateringStatus: shared.WateringStatusUnknown,
		Latitude:       54.807162,
		Longitude:      9.423138,
		SoilCondition:  cluster.TreeSoilConditionSandig,
		RegionID:       13,
		TreeIDs:        []int32{21, 22, 23, 24},
	},
	{
		ID:             7,
		Name:           "TSB Neustadt",
		Address:        "Ecknerstraße",
		Description:    "Kleiner Baumbestand.",
		MoistureLevel:  0.75,
		WateringStatus: shared.WateringStatusGood,
		Latitude:       54.797162,
		Longitude:      9.41962,
		SoilCondition:  cluster.TreeSoilConditionSandig,
		RegionID:       13,
	},
	{
		ID:             8,
		Name:           "Gewerbegebiet Süd",
		Address:        "Address8",
		Description:    "Description8",
		MoistureLevel:  8.0,
		WateringStatus: shared.WateringStatusGood,
		Latitude:       0xc0012fc3d0,
		Longitude:      0xc0012fc3d8,
		SoilCondition:  cluster.TreeSoilConditionLehmig,
		RegionID:       -1, // no region
	},
	{
		ID:             50,
		Name:           "Gewerbegebiet Süd",
		Address:        "Address9",
		Description:    "Description9",
		MoistureLevel:  9.0,
		WateringStatus: shared.WateringStatusGood,
		Latitude:       0xc0011fc3d0,
		Longitude:      0xc0011fc3d8,
		SoilCondition:  cluster.TreeSoilConditionLehmig,
		RegionID:       -1, // no region
		TreeIDs:        []int32{25, 26, 27, 28},
		Provider:       "test-provider",
		AdditionalInfo: map[string]interface{}{
			"foo": "bar",
		},
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

