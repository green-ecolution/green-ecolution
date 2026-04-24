package tree_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/utils"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	httpEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
)

var (
	testLatitude          = 9.446741
	testLongitude         = 54.801539
	TestTreeUpdateRequest = (*httpEntities.TreeUpdateRequest)(getMockTreeRequest("Updated description"))
	TestTreeCreateRequest = getMockTreeRequest("Created description")
	TestTrees             = []*treeDomain.Tree{
		{
			ID:           1,
			CreatedAt:    time.Now(),
			UpdatedAt:    time.Now(),
			Species:      "Oak",
			Number:       "T001",
			Coordinate:   shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:  "A mature oak tree",
			PlantingYear: treeDomain.MustNewPlantingYear(2023),
		},
		{
			ID:           2,
			CreatedAt:    time.Now(),
			UpdatedAt:    time.Now(),
			Species:      "Pine",
			Number:       "T002",
			Coordinate:   shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:  "A young pine tree",
			PlantingYear: treeDomain.MustNewPlantingYear(2023),
		},
	}

	testClusterID = utils.P(int32(1))

	testFilterTrees = []*treeDomain.Tree{
		{
			ID:             1,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Oak",
			Number:         "T001",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			Description:    "A mature oak tree",
			TreeClusterID:  testClusterID,
			WateringStatus: shared.WateringStatusGood,
			PlantingYear:   treeDomain.MustNewPlantingYear(2023),
		},
		{
			ID:             2,
			CreatedAt:      time.Now(),
			UpdatedAt:      time.Now(),
			Species:        "Pine",
			Number:         "T002",
			Coordinate:     shared.MustNewCoordinate(testLatitude, testLongitude),
			TreeClusterID:  testClusterID,
			Description:    "A young pine tree",
			WateringStatus: shared.WateringStatusBad,
			PlantingYear:   treeDomain.MustNewPlantingYear(2022),
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
