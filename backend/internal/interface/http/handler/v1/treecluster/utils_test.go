package treecluster_test

import (
	"time"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	testLatitude  = 9.446741
	testLongitude = 54.801539

	TestCluster = &domain.TreeCluster{
		ID:             1,
		Name:           "Test Cluster",
		Address:        "456 New St",
		Description:    "Description",
		WateringStatus: domain.WateringStatusBad,
		Region:         &domain.Region{ID: 1, Name: "Region 1"},
		Archived:       false,
		Coordinate:     utils.P(domain.MustNewCoordinate(testLatitude, testLongitude)),
		SoilCondition:  domain.TreeSoilConditionSandig,
		Trees: []*domain.Tree{
			{
				ID:           1,
				CreatedAt:    time.Now(),
				UpdatedAt:    time.Now(),
				Species:      "Oak",
				Number:       "T001",
				Coordinate:   domain.MustNewCoordinate(testLatitude, testLongitude),
				Description:  "A mature oak tree",
				PlantingYear: domain.MustNewPlantingYear(2023),
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

	TestClusterList = []*domain.TreeCluster{
		TestCluster,
		{
			ID:             2,
			Name:           "Second Cluster",
			Address:        "789 Another St",
			Description:    "Another description",
			WateringStatus: domain.WateringStatusGood,
			Region:         &domain.Region{ID: 2, Name: "Region 2"},
			Archived:       false,
			Coordinate:     utils.P(domain.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  domain.TreeSoilConditionLehmig,
			Trees:          []*domain.Tree{},
		},
		{
			ID:             3,
			Name:           "Third Cluster",
			Address:        "101 Forest Rd",
			Description:    "Forest description",
			WateringStatus: domain.WateringStatusModerate,
			Region:         &domain.Region{ID: 1, Name: "Mürwik"},
			Archived:       false,
			Coordinate:     utils.P(domain.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  domain.TreeSoilConditionSandig,
			Trees:          []*domain.Tree{},
		},
		{
			ID:             4,
			Name:           "Fourth Cluster",
			Address:        "15 Lake Side",
			Description:    "Near a lake",
			WateringStatus: domain.WateringStatusBad,
			Region:         &domain.Region{ID: 3, Name: "Mürwik"},
			Archived:       false,
			Coordinate:     utils.P(domain.MustNewCoordinate(testLatitude, testLongitude)),
			SoilCondition:  domain.TreeSoilConditionLehmig,
			Trees:          []*domain.Tree{},
		},
	}
)
