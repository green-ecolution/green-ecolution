package tree_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	httpEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
)

var (
	testLatitude          = 9.446741
	testLongitude         = 54.801539
	TestTreeUpdateRequest = (*httpEntities.TreeUpdateRequest)(getMockTreeRequest("Updated description"))
	TestTreeCreateRequest = getMockTreeRequest("Created description")
	TestTrees             = []*shared.Tree{
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
		{
			ID:           2,
			CreatedAt:    time.Now(),
			UpdatedAt:    time.Now(),
			Species:      "Pine",
			Number:       "T002",
			Coordinate:   shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:  "A young pine tree",
			PlantingYear: shared.MustNewPlantingYear(2023),
		},
	}

	testCluster = &shared.TreeCluster{
		ID:             1,
		Name:           "Test Cluster",
		Address:        "456 New St",
		Description:    "Description",
		WateringStatus: shared.WateringStatusBad,
		Region:         &shared.Region{ID: 1, Name: "Region 1"},
		Archived:       false,
		Coordinate:     utils.P(shared.MustNewCoordinate(testLatitude, testLongitude)),
		SoilCondition:  shared.TreeSoilConditionSandig,
	}

	testFilterTrees = []*shared.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			TreeCluster:    testCluster,
			WateringStatus: shared.WateringStatusGood,
			PlantingYear:   shared.MustNewPlantingYear(2023),
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			TreeCluster:    testCluster,
			Description:    "A young pine tree",
			WateringStatus: shared.WateringStatusBad,
			PlantingYear:   shared.MustNewPlantingYear(2022),
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
