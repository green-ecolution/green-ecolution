package treecluster_test

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	clusterDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	testLatitude  = 9.446741
	testLongitude = 54.801539

	TestCluster = &clusterDomain.TreeCluster{
		ID:             1,
		Name:           "Test Cluster",
		Address:        "456 New St",
		Description:    "Description",
		WateringStatus: shared.WateringStatusBad,
		RegionID:       utils.P(int32(1)),
		Archived:       false,
		Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
		SoilCondition:  cluster.TreeSoilConditionSandig,
		TreeIDs:        []int32{1},
	}

	TestClusterRequest = serverEntities.TreeClusterCreateRequest{
		Name:          "Cluster Request",
		Address:       "123 Main St",
		Description:   "Test description",
		SoilCondition: serverEntities.TreeSoilConditionSandig,
		TreeIDs:       []*int32{utils.P(int32(1))},
	}

	TestClusterList = []*clusterDomain.TreeCluster{
		TestCluster,
		{
			ID:             2,
			Name:           "Second Cluster",
			Address:        "789 Another St",
			Description:    "Another description",
			WateringStatus: shared.WateringStatusGood,
			RegionID:       utils.P(int32(2)),
			Archived:       false,
			Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  cluster.TreeSoilConditionLehmig,
			TreeIDs:        []int32{},
		},
		{
			ID:             3,
			Name:           "Third Cluster",
			Address:        "101 Forest Rd",
			Description:    "Forest description",
			WateringStatus: shared.WateringStatusModerate,
			RegionID:       utils.P(int32(1)),
			Archived:       false,
			Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  cluster.TreeSoilConditionSandig,
			TreeIDs:        []int32{},
		},
		{
			ID:             4,
			Name:           "Fourth Cluster",
			Address:        "15 Lake Side",
			Description:    "Near a lake",
			WateringStatus: shared.WateringStatusBad,
			RegionID:       utils.P(int32(3)),
			Archived:       false,
			Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  cluster.TreeSoilConditionLehmig,
			TreeIDs:        []int32{},
		},
	}
)
