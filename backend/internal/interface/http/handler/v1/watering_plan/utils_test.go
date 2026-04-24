package wateringplan_test

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

var (
	TestWateringPlans = []*watering.WateringPlan{
		{
			ID:                 1,
			Date:               time.Date(2024, 9, 22, 0, 0, 0, 0, time.UTC),
			Description:        "New watering plan for the west side of the city",
			Status:             watering.WateringPlanStatusPlanned,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
			TotalWaterRequired: utils.P(6000.0),
			TransporterID:      utils.P(int32(2)),
			TrailerID:          utils.P(int32(1)),
			TreeClusterIDs:     []int32{1, 2},
			CancellationNote:   "",
		},
		{
			ID:                 2,
			Date:               time.Date(2024, 8, 3, 0, 0, 0, 0, time.UTC),
			Description:        "New watering plan for the east side of the city",
			Status:             watering.WateringPlanStatusActive,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
			TotalWaterRequired: utils.P(6000.0),
			TransporterID:      utils.P(int32(2)),
			TrailerID:          utils.P(int32(1)),
			TreeClusterIDs:     []int32{3},
			CancellationNote:   "",
		},
		{
			ID:                 3,
			Date:               time.Date(2024, 6, 12, 0, 0, 0, 0, time.UTC),
			Description:        "Very important watering plan due to no rainfall",
			Status:             watering.WateringPlanStatusFinished,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
			TotalWaterRequired: utils.P(6000.0),
			TransporterID:      utils.P(int32(2)),
			TrailerID:          nil,
			TreeClusterIDs:     []int32{1, 2, 3},
			CancellationNote:   "",
			Evaluation: []*watering.EvaluationValue{
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
			Status:             watering.WateringPlanStatusNotCompeted,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
			TotalWaterRequired: utils.P(6000.0),
			TransporterID:      utils.P(int32(2)),
			TrailerID:          nil,
			TreeClusterIDs:     []int32{3},
			CancellationNote:   "",
		},
		{
			ID:                 5,
			Date:               time.Date(2024, 6, 4, 0, 0, 0, 0, time.UTC),
			Description:        "Canceled due to flood",
			Status:             watering.WateringPlanStatusCanceled,
			Distance:           utils.P(shared.MustNewDistance(63.0)),
			TotalWaterRequired: utils.P(6000.0),
			TransporterID:      utils.P(int32(2)),
			TrailerID:          nil,
			TreeClusterIDs:     []int32{3},
			CancellationNote:   "The watering plan was cancelled due to various reasons.",
		},
	}
	TestClusters = []*cluster.TreeCluster{
		{
			ID:             1,
			Name:           "Solitüde Strand",
			WateringStatus: shared.WateringStatusGood,
			MoistureLevel:  0.75,
			RegionID:       utils.P(int32(1)),
			Address:        "Solitüde Strand",
			Description:    "Alle Bäume am Strand",
			SoilCondition:  cluster.TreeSoilConditionSandig,
			Coordinate:     utils.P(shared.MustNewCoordinate(54.820940, 9.489022)),
			TreeIDs:        []int32{1, 2, 3},
		},
		{
			ID:             2,
			Name:           "Sankt-Jürgen-Platz",
			WateringStatus: shared.WateringStatusModerate,
			MoistureLevel:  0.5,
			RegionID:       utils.P(int32(1)),
			Address:        "Ulmenstraße",
			Description:    "Bäume beim Sankt-Jürgen-Platz",
			SoilCondition:  cluster.TreeSoilConditionSchluffig,
			Coordinate:     utils.P(shared.MustNewCoordinate(54.78805731048199, 9.44400186680097)),
			TreeIDs:        []int32{4, 5, 6},
		},
		{
			ID:             3,
			Name:           "Flensburger Stadion",
			WateringStatus: "unknown",
			MoistureLevel:  0.7,
			RegionID:       utils.P(int32(1)),
			Address:        "Flensburger Stadion",
			Description:    "Alle Bäume in der Gegend des Stadions in Mürwik",
			SoilCondition:  "schluffig",
			Coordinate:     utils.P(shared.MustNewCoordinate(54.802163, 9.446398)),
			TreeIDs:        []int32{},
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
