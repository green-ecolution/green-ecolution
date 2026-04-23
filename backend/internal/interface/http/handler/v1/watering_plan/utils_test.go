package wateringplan_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	TestWateringPlans = []*shared.WateringPlan{
		{
			ID:                 1,
			Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
			Description:        "New watering plan for the west side of the city",
			Status:             shared.WateringPlanStatusPlanned,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
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
			Status:             shared.WateringPlanStatusActive,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
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
			Status:             shared.WateringPlanStatusFinished,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
			TotalWaterRequired: utils.P(6000.0),
			Transporter:        TestVehicles[1],
			Trailer:            nil,
			TreeClusters:       TestClusters[0:3],
			CancellationNote:   "",
			Evaluation: []*shared.EvaluationValue{
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
			Status:             shared.WateringPlanStatusNotCompeted,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
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
			Status:             shared.WateringPlanStatusCanceled,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
			TotalWaterRequired: utils.P(6000.0),
			Transporter:        TestVehicles[1],
			Trailer:            nil,
			TreeClusters:       TestClusters[2:3],
			CancellationNote:   "The watering plan was cancelled due to various reasons.",
		},
	}
	TestVehicles = []*shared.Vehicle{
		{
			ID:            1,
			NumberPlate:   "B-1234",
			Description:   "Test vehicle 1",
			WaterCapacity: shared.MustNewWaterCapacity(100.0),
			Type:          shared.VehicleTypeTrailer,
			Status:        shared.VehicleStatusActive,
		},
		{
			ID:            2,
			NumberPlate:   "B-5678",
			Description:   "Test vehicle 2",
			WaterCapacity: shared.MustNewWaterCapacity(150.0),
			Type:          shared.VehicleTypeTransporter,
			Status:        shared.VehicleStatusUnknown,
		},
	}
	TestClusters = []*shared.TreeCluster{
		{
			ID:             1,
			Name:           "Solitüde Strand",
			WateringStatus: shared.WateringStatusGood,
			MoistureLevel:  0.75,
			Region: &shared.Region{
				ID:   1,
				Name: "Mürwik",
			},
			Address:       "Solitüde Strand",
			Description:   "Alle Bäume am Strand",
			SoilCondition: shared.TreeSoilConditionSandig,
			Coordinate:    utils.P(shared.MustNewCoordinate(54.820940, 9.489022)),
			Trees: []*shared.Tree{
				{ID: 1},
				{ID: 2},
				{ID: 3},
			},
		},
		{
			ID:             2,
			Name:           "Sankt-Jürgen-Platz",
			WateringStatus: shared.WateringStatusModerate,
			MoistureLevel:  0.5,
			Region: &shared.Region{
				ID:   1,
				Name: "Mürwik",
			},
			Address:       "Ulmenstraße",
			Description:   "Bäume beim Sankt-Jürgen-Platz",
			SoilCondition: shared.TreeSoilConditionSchluffig,
			Coordinate:    utils.P(shared.MustNewCoordinate(54.78805731048199, 9.44400186680097)),
			Trees: []*shared.Tree{
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
			Region: &shared.Region{
				ID:   1,
				Name: "Mürwik",
			},
			Address:       "Flensburger Stadion",
			Description:   "Alle Bäume in der Gegend des Stadions in Mürwik",
			SoilCondition: "schluffig",
			Coordinate:    utils.P(shared.MustNewCoordinate(54.802163, 9.446398)),
			Trees:         []*shared.Tree{},
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
