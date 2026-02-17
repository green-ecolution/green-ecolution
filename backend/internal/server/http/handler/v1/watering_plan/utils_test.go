package wateringplan_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	TestWateringPlans = []*entities.WateringPlan{
		{
			ID:                 1,
			Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
			Description:        "New watering plan for the west side of the city",
			Status:             entities.WateringPlanStatusPlanned,
			Distance:           utils.P(63.0),
			TotalWaterRequired: utils.P(6000.0),
			Transporter:        TestVehicles[1],
			Trailer:            TestVehicles[0],
			TreeClusters:       TestClusters[0:2],
			CancellationNote:   "",
		},
		{
			ID:                 2,
			Date:               time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			Description:        "New watering plan for the east side of the city",
			Status:             entities.WateringPlanStatusActive,
			Distance:           utils.P(63.0),
			TotalWaterRequired: utils.P(6000.0),
			Transporter:        TestVehicles[1],
			Trailer:            TestVehicles[0],
			TreeClusters:       TestClusters[2:3],
			CancellationNote:   "",
		},
		{
			ID:                 3,
			Date:               time.Date(2024, 6, 12, 0, 0, 0, 0, time.UTC),
			Description:        "Very important watering plan due to no rainfall",
			Status:             entities.WateringPlanStatusFinished,
			Distance:           utils.P(63.0),
			TotalWaterRequired: utils.P(6000.0),
			Transporter:        TestVehicles[1],
			Trailer:            nil,
			TreeClusters:       TestClusters[0:3],
			CancellationNote:   "",
			Evaluation: []*entities.EvaluationValue{
				{
					WateringPlanID: 3,
					TreeClusterID:  1,
					ConsumedWater:  utils.P(10.0),
				},
				{
					WateringPlanID: 3,
					TreeClusterID:  2,
					ConsumedWater:  utils.P(10.0),
				},
				{
					WateringPlanID: 3,
					TreeClusterID:  3,
					ConsumedWater:  utils.P(10.0),
				},
			},
		},
		{
			ID:                 4,
			Date:               time.Date(2024, 6, 10, 0, 0, 0, 0, time.UTC),
			Description:        "New watering plan for the south side of the city",
			Status:             entities.WateringPlanStatusNotCompeted,
			Distance:           utils.P(63.0),
			TotalWaterRequired: utils.P(6000.0),
			Transporter:        TestVehicles[1],
			Trailer:            nil,
			TreeClusters:       TestClusters[2:3],
			CancellationNote:   "",
		},
		{
			ID:                 5,
			Date:               time.Date(2024, 6, 4, 0, 0, 0, 0, time.UTC),
			Description:        "Canceled due to flood",
			Status:             entities.WateringPlanStatusCanceled,
			Distance:           utils.P(63.0),
			TotalWaterRequired: utils.P(6000.0),
			Transporter:        TestVehicles[1],
			Trailer:            nil,
			TreeClusters:       TestClusters[2:3],
			CancellationNote:   "The watering plan was cancelled due to various reasons.",
		},
	}
	TestVehicles = []*entities.Vehicle{
		{
			ID:            1,
			NumberPlate:   "B-1234",
			Description:   "Test vehicle 1",
			WaterCapacity: 100.0,
			Type:          entities.VehicleTypeTrailer,
			Status:        entities.VehicleStatusActive,
		},
		{
			ID:            2,
			NumberPlate:   "B-5678",
			Description:   "Test vehicle 2",
			WaterCapacity: 150.0,
			Type:          entities.VehicleTypeTransporter,
			Status:        entities.VehicleStatusUnknown,
		},
	}
	TestClusters = []*entities.TreeCluster{
		{
			ID:             1,
			Name:           "Solitüde Strand",
			WateringStatus: entities.WateringStatusGood,
			MoistureLevel:  0.75,
			Region: &entities.Region{
				ID:   1,
				Name: "Mürwik",
			},
			Address:       "Solitüde Strand",
			Description:   "Alle Bäume am Strand",
			SoilCondition: entities.TreeSoilConditionSandig,
			Latitude:      utils.P(54.820940),
			Longitude:     utils.P(9.489022),
			Trees: []*entities.Tree{
				{ID: 1},
				{ID: 2},
				{ID: 3},
			},
		},
		{
			ID:             2,
			Name:           "Sankt-Jürgen-Platz",
			WateringStatus: entities.WateringStatusModerate,
			MoistureLevel:  0.5,
			Region: &entities.Region{
				ID:   1,
				Name: "Mürwik",
			},
			Address:       "Ulmenstraße",
			Description:   "Bäume beim Sankt-Jürgen-Platz",
			SoilCondition: entities.TreeSoilConditionSchluffig,
			Latitude:      utils.P(54.78805731048199),
			Longitude:     utils.P(9.44400186680097),
			Trees: []*entities.Tree{
				{ID: 4},
				{ID: 5},
				{ID: 6},
			},
		},
		{
			ID:             3,
			Name:           "Flensburger Stadion",
			WateringStatus: "unknown",
			MoistureLevel:  0.7,
			Region: &entities.Region{
				ID:   1,
				Name: "Mürwik",
			},
			Address:       "Flensburger Stadion",
			Description:   "Alle Bäume in der Gegend des Stadions in Mürwik",
			SoilCondition: "schluffig",
			Latitude:      utils.P(54.802163),
			Longitude:     utils.P(9.446398),
			Trees:         []*entities.Tree{},
		},
	}

	TestWateringPlanRequest = &serverEntities.WateringPlanCreateRequest{
		Date:           time.Now().Add(24 * time.Hour).Truncate(24 * time.Hour),
		Description:    "New watering plan for the west side of the city",
		TransporterID:  utils.P(int32(1)),
		TrailerID:      utils.P(int32(2)),
		TreeClusterIDs: []*int32{utils.P(int32(1)), utils.P(int32(2))},
		UserIDs:        []string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"},
	}

	TestWateringPlanUpdateRequest = &serverEntities.WateringPlanUpdateRequest{
		Date:           time.Now().Add(24 * time.Hour).Truncate(24 * time.Hour),
		Description:    "Updated watering plan for the west side of the city",
		TransporterID:  utils.P(int32(1)),
		TrailerID:      utils.P(int32(2)),
		TreeClusterIDs: []*int32{utils.P(int32(1)), utils.P(int32(2))},
		UserIDs:        []string{"6a1078e8-80fd-458f-b74e-e388fe2dd6ab"},
		Status:         serverEntities.WateringPlanStatusPlanned,
	}
)
