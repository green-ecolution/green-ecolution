package treecluster_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	testLatitude  = 9.446741
	testLongitude = 54.801539

	TestCluster = &shared.TreeCluster{
		ID:             1,
		Name:           "Test Cluster",
		Address:        "456 New St",
		Description:    "Description",
		WateringStatus: shared.WateringStatusBad,
		Region:         &shared.Region{ID: 1, Name: "Region 1"},
		Archived:       false,
		Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
		SoilCondition:  shared.TreeSoilConditionSandig,
		Trees: []*shared.Tree{
			{
				ID:           1,
				CreatedAt:    time.Now(),
				UpdatedAt:    time.Now(),
				Species:      "Oak",
				Number:       "T001",
				Coordinate:   shared.MustNewCoordinate(testLatitude, testLongitude),
				Description:  "A mature oak tree",
				PlantingYear: shared.MustNewPlantingYear(2023),
			},
		},
	}

	TestClusterRequest = serverEntities.TreeClusterCreateRequest{
		Name:          "Cluster Request",
		Address:       "123 Main St",
		Description:   "Test description",
		SoilCondition: serverEntities.TreeSoilConditionSandig,
		TreeIDs:       []*int32{utils.P(int32(1))},
	}

	TestClusterList = []*shared.TreeCluster{
		TestCluster,
		{
			ID:             2,
			Name:           "Second Cluster",
			Address:        "789 Another St",
			Description:    "Another description",
			WateringStatus: shared.WateringStatusGood,
			Region:         &shared.Region{ID: 2, Name: "Region 2"},
			Archived:       false,
			Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  shared.TreeSoilConditionLehmig,
			Trees:          []*shared.Tree{},
		},
		{
			ID:             3,
			Name:           "Third Cluster",
			Address:        "101 Forest Rd",
			Description:    "Forest description",
			WateringStatus: shared.WateringStatusModerate,
			Region:         &shared.Region{ID: 1, Name: "Mürwik"},
			Archived:       false,
			Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  shared.TreeSoilConditionSandig,
			Trees:          []*shared.Tree{},
		},
		{
			ID:             4,
			Name:           "Fourth Cluster",
			Address:        "15 Lake Side",
			Description:    "Near a lake",
			WateringStatus: shared.WateringStatusBad,
			Region:         &shared.Region{ID: 3, Name: "Mürwik"},
			Archived:       false,
			Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  shared.TreeSoilConditionLehmig,
			Trees:          []*shared.Tree{},
		},
	}
)
