package treecluster_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	testLatitude  = 9.446741
	testLongitude = 54.801539

	TestCluster = &entities.TreeCluster{
		ID:             1,
		Name:           "Test Cluster",
		Address:        "456 New St",
		Description:    "Description",
		WateringStatus: entities.WateringStatusBad,
		Region:         &entities.Region{ID: 1, Name: "Region 1"},
		Archived:       false,
		Latitude:       utils.P(testLatitude),
		Longitude:      utils.P(testLongitude),
		SoilCondition:  entities.TreeSoilConditionSandig,
		Trees: []*entities.Tree{
			{
				ID:           1,
				CreatedAt:    time.Now(),
				UpdatedAt:    time.Now(),
				Species:      "Oak",
				Number:       "T001",
				Latitude:     testLatitude,
				Longitude:    testLongitude,
				Description:  "A mature oak tree",
				PlantingYear: 2023,
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

	TestClusterList = []*entities.TreeCluster{
		TestCluster,
		{
			ID:             2,
			Name:           "Second Cluster",
			Address:        "789 Another St",
			Description:    "Another description",
			WateringStatus: entities.WateringStatusGood,
			Region:         &entities.Region{ID: 2, Name: "Region 2"},
			Archived:       false,
			Latitude:       utils.P(testLatitude),
			Longitude:      utils.P(testLongitude),
			SoilCondition:  entities.TreeSoilConditionLehmig,
			Trees:          []*entities.Tree{},
		},
		{
			ID:             3,
			Name:           "Third Cluster",
			Address:        "101 Forest Rd",
			Description:    "Forest description",
			WateringStatus: entities.WateringStatusModerate,
			Region:         &entities.Region{ID: 1, Name: "Mürwik"},
			Archived:       false,
			Latitude:       utils.P(testLatitude),
			Longitude:      utils.P(testLongitude),
			SoilCondition:  entities.TreeSoilConditionSandig,
			Trees:          []*entities.Tree{},
		},
		{
			ID:             4,
			Name:           "Fourth Cluster",
			Address:        "15 Lake Side",
			Description:    "Near a lake",
			WateringStatus: entities.WateringStatusBad,
			Region:         &entities.Region{ID: 3, Name: "Mürwik"},
			Archived:       false,
			Latitude:       utils.P(testLatitude),
			Longitude:      utils.P(testLongitude),
			SoilCondition:  entities.TreeSoilConditionLehmig,
			Trees:          []*entities.Tree{},
		},
	}
)
