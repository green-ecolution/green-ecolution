package tree_test

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	httpEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

var (
	testLatitude          = 9.446741
	testLongitude         = 54.801539
	TestTreeUpdateRequest = (*httpEntities.TreeUpdateRequest)(getMockTreeRequest("Updated description"))
	TestTreeCreateRequest = getMockTreeRequest("Created description")
	TestTrees             = []*entities.Tree{
		{
			ID:           1,
			CreatedAt:    time.Now(),
			UpdatedAt:    time.Now(),
			Species:      "Oak",
			Number:       "T001",
			Coordinate:   entities.MustNewCoordinate(testLatitude, testLongitude),
			Description:  "A mature oak tree",
			PlantingYear: entities.MustNewPlantingYear(2023),
		},
		{
			ID:           2,
			CreatedAt:    time.Now(),
			UpdatedAt:    time.Now(),
			Species:      "Pine",
			Number:       "T002",
			Coordinate:   entities.MustNewCoordinate(testLatitude, testLongitude),
			Description:  "A young pine tree",
			PlantingYear: entities.MustNewPlantingYear(2023),
		},
	}

	testCluster = &entities.TreeCluster{
		ID:             1,
		Name:           "Test Cluster",
		Address:        "456 New St",
		Description:    "Description",
		WateringStatus: entities.WateringStatusBad,
		Region:         &entities.Region{ID: 1, Name: "Region 1"},
		Archived:       false,
		Coordinate:     utils.P(entities.MustNewCoordinate(testLatitude, testLongitude)),
		SoilCondition:  entities.TreeSoilConditionSandig,
	}

	testFilterTrees = []*entities.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     entities.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			TreeCluster:    testCluster,
			WateringStatus: entities.WateringStatusGood,
			PlantingYear:   entities.MustNewPlantingYear(2023),
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     entities.MustNewCoordinate(testLatitude, testLongitude),
			TreeCluster:    testCluster,
			Description:    "A young pine tree",
			WateringStatus: entities.WateringStatusBad,
			PlantingYear:   entities.MustNewPlantingYear(2022),
		},
	}
)

func getMockTreeRequest(description string) *httpEntities.TreeCreateRequest {
	return &httpEntities.TreeCreateRequest{
		TreeClusterID: nil,
		PlantingYear:  2023,
		Species:       "Oak",
		Number:        "T001",
		Latitude:      testLatitude,
		Longitude:     testLongitude,
		SensorID:      nil,
		Description:   description,
	}
}
