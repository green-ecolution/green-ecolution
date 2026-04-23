package tree_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	httpEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
)

var (
	testLatitude          = 9.446741
	testLongitude         = 54.801539
	TestTreeUpdateRequest = (*httpEntities.TreeUpdateRequest)(getMockTreeRequest("Updated description"))
	TestTreeCreateRequest = getMockTreeRequest("Created description")
	TestTrees             = []*domain.Tree{
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
		{
			ID:           2,
			CreatedAt:    time.Now(),
			UpdatedAt:    time.Now(),
			Species:      "Pine",
			Number:       "T002",
			Coordinate:   domain.MustNewCoordinate(testLatitude, testLongitude),
			Description:  "A young pine tree",
			PlantingYear: domain.MustNewPlantingYear(2023),
		},
	}

	testCluster = &domain.TreeCluster{
		ID:             1,
		Name:           "Test Cluster",
		Address:        "456 New St",
		Description:    "Description",
		WateringStatus: domain.WateringStatusBad,
		Region:         &domain.Region{ID: 1, Name: "Region 1"},
		Archived:       false,
		Coordinate:     utils.P(domain.MustNewCoordinate(testLatitude, testLongitude)),
		SoilCondition:  domain.TreeSoilConditionSandig,
	}

	testFilterTrees = []*domain.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     domain.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			TreeCluster:    testCluster,
			WateringStatus: domain.WateringStatusGood,
			PlantingYear:   domain.MustNewPlantingYear(2023),
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     domain.MustNewCoordinate(testLatitude, testLongitude),
			TreeCluster:    testCluster,
			Description:    "A young pine tree",
			WateringStatus: domain.WateringStatusBad,
			PlantingYear:   domain.MustNewPlantingYear(2022),
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
