package wateringplan

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func defaultWateringPlan() *entities.WateringPlan {
	return &entities.WateringPlan{
		Date:               time.Time{},
		Description:        "",
		Distance:           utils.P(0.0),
		TotalWaterRequired: utils.P(0.0),
		Status:             entities.WateringPlanStatusPlanned,
		UserIDs:            make([]*uuid.UUID, 0),
		TreeClusters:       make([]*entities.TreeCluster, 0),
		Transporter:        nil,
		Trailer:            nil,
		CancellationNote:   "",
		Duration:           time.Duration(0),
		RefillCount:        0,
		Provider:           "",
		AdditionalInfo:     nil,
	}
}

func (w *WateringPlanRepository) Create(ctx context.Context, createFn func(*entities.WateringPlan, storage.WateringPlanRepository) (bool, error)) (*entities.WateringPlan, error) {
	log := logger.GetLogger(ctx)
	if createFn == nil {
		return nil, errors.New("createFn is nil")
	}

	var createdWp *entities.WateringPlan
	err := w.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewWateringPlanRepository(s, w.WateringPlanMappers)
		entity := defaultWateringPlan()
		created, err := createFn(entity, newRepo)
		if err != nil {
			return err
		}

		if !created {
			return nil
		}

		if err := newRepo.validateWateringPlan(entity); err != nil {
			return err
		}

		id, err := newRepo.createEntity(ctx, entity)
		if err != nil {
			return err
		}

		createdWp, err = newRepo.GetByID(ctx, *id)
		if err != nil {
			fmt.Println("failed to get vy id")
			return err
		}

		return nil
	})

	if err != nil {
		log.Error("failed to create watering plan entity in db", "error", err)
		return nil, err
	}

	if createdWp != nil {
		log.Debug("tree cluster entity created successfully", "cluster_id", createdWp.ID)
	} else {
		log.Debug("tree cluster should not be created. cancel transaction")
	}
	return createdWp, nil
}

func (w *WateringPlanRepository) createEntity(ctx context.Context, entity *entities.WateringPlan) (*int32, error) {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(entity.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", entity.AdditionalInfo)
		return nil, err
	}

	// this resembles the old behavior for zero dates
	if entity.Date.IsZero() {
		return nil, errors.New("failed to convert date")
	}

	args := sqlc.CreateWateringPlanParams{
		Date:                   entity.Date,
		Description:            entity.Description,
		Distance:               entity.Distance,
		TotalWaterRequired:     entity.TotalWaterRequired,
		Status:                 sqlc.WateringPlanStatus(entities.WateringPlanStatusPlanned),
		Provider:               &entity.Provider,
		AdditionalInformations: additionalInfo,
	}

	id, err := w.store.CreateWateringPlan(ctx, &args)
	if err != nil {
		log.Debug("failed to create watering plan in createEntity func", "error", err)
		return nil, err
	}

	if err := w.setLinkedUsers(ctx, entity, id); err != nil {
		log.Debug("failed to link users to currently created watering plan", "error", err, "user_ids", entity.UserIDs, "watering_plan_id", id)
		return nil, err
	}

	if err := w.setLinkedVehicles(ctx, entity, id); err != nil {
		if entity.Trailer != nil {
			log.Debug("failed to link vehicles to currently created watering plan", "error", err, "transporter_id", entity.Transporter.ID, "trailer_id", entity.Trailer.ID, "watering_plan_id", id)
		} else {
			log.Debug("failed to link vehicles to currently created watering plan", "error", err, "transporter_id", entity.Transporter.ID, "watering_plan_id", id)
		}
		return nil, err
	}

	if err := w.setLinkedTreeClusters(ctx, entity, id); err != nil {
		if entity.TreeClusters != nil {
			log.Debug("failed to link tree cluster to currently created watering plan", "error", err, "cluster_ids", utils.Map(entity.TreeClusters, func(c *entities.TreeCluster) int32 { return c.ID }), "watering_plan_id", id)
		}
		return nil, err
	}

	return &id, nil
}

func (w *WateringPlanRepository) validateWateringPlan(entity *entities.WateringPlan) error {
	if entity.Transporter == nil || entity.Transporter.Type != entities.VehicleTypeTransporter {
		return errors.New("watering plan requires a valid transporter")
	}

	if entity.Trailer != nil && entity.Trailer.Type != entities.VehicleTypeTrailer {
		return errors.New("trailer vehicle requires a vehicle of type trailer")
	}

	if len(entity.UserIDs) == 0 {
		return errors.New("watering plan requires employees")
	}

	if len(entity.TreeClusters) == 0 {
		return errors.New("watering plan requires tree cluster")
	}

	return nil
}

func (w *WateringPlanRepository) setLinkedVehicles(ctx context.Context, entity *entities.WateringPlan, id int32) error {
	log := logger.GetLogger(ctx)
	// link transporter to pivot table
	err := w.store.SetVehicleToWateringPlan(ctx, &sqlc.SetVehicleToWateringPlanParams{
		WateringPlanID: id,
		VehicleID:      entity.Transporter.ID,
	})
	if err != nil {
		log.Error("failed to link transporter vehicle to watering plan", "error", err, "watering_plan_id", id, "vehicle_id", entity.Transporter.ID)
		return err
	}

	// link trailer to pivot table
	if entity.Trailer != nil {
		err = w.store.SetVehicleToWateringPlan(ctx, &sqlc.SetVehicleToWateringPlanParams{
			WateringPlanID: id,
			VehicleID:      entity.Trailer.ID,
		})
		if err != nil {
			log.Error("failed to link trailer vehicle to watering plan", "error", err, "watering_plan_id", id, "vehicle_id", entity.Trailer.ID)
			return err
		}
	}

	return nil
}

func (w *WateringPlanRepository) setLinkedUsers(ctx context.Context, entity *entities.WateringPlan, id int32) error {
	log := logger.GetLogger(ctx)
	for _, userID := range entity.UserIDs {
		err := w.store.SetUserToWateringPlan(ctx, &sqlc.SetUserToWateringPlanParams{
			WateringPlanID: id,
			UserID:         *userID,
		})
		if err != nil {
			log.Error("failed to link users to watering plan", "error", err, "watering_plan_id", id, "user_ids", entity.UserIDs)
			return err
		}
	}

	return nil
}

func (w *WateringPlanRepository) setLinkedTreeClusters(ctx context.Context, entity *entities.WateringPlan, id int32) error {
	log := logger.GetLogger(ctx)
	for _, tc := range entity.TreeClusters {
		err := w.store.SetTreeClusterToWateringPlan(ctx, &sqlc.SetTreeClusterToWateringPlanParams{
			WateringPlanID: id,
			TreeClusterID:  tc.ID,
		})
		if err != nil {
			log.Error("failed to link tree clusters to watering plan", "error", err, "watering_plan_id", id, "cluster_ids", utils.Map(entity.TreeClusters, func(c *entities.TreeCluster) int32 { return c.ID }))
			return err
		}
	}

	return nil
}
