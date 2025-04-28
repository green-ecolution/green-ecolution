package wateringplan

import (
	"context"
	"errors"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/backend/internal/utils"
)

func (w *WateringPlanRepository) Update(ctx context.Context, id int32, updateFn func(*entities.WateringPlan, storage.WateringPlanRepository) (bool, error)) error {
	log := logger.GetLogger(ctx)
	return w.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewWateringPlanRepository(s, w.WateringPlanMappers)
		entity, err := newRepo.GetByID(ctx, id)
		if err != nil {
			return err
		}

		if updateFn == nil {
			return errors.New("updateFn is nil")
		}

		updated, err := updateFn(entity, newRepo)
		if err != nil {
			return err
		}

		if !updated {
			return nil
		}

		if err := newRepo.validateWateringPlan(entity); err != nil {
			return err
		}

		if err := newRepo.updateEntity(ctx, entity); err != nil {
			log.Error("failed to updated watering plan entity in db", "error", err, "watering_plan_id", id)
			return err
		}

		log.Debug("watering plan entity updated successfully", "watering_plan_id", id)
		return nil
	})
}

func (w *WateringPlanRepository) updateEntity(ctx context.Context, entity *entities.WateringPlan) error {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(entity.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", entity.AdditionalInfo)
		return err
	}

	date, err := utils.TimeToPgDate(entity.Date)
	if err != nil {
		return errors.New("failed to convert date")
	}

	if entity.CancellationNote != "" && entity.Status != entities.WateringPlanStatusCanceled {
		return errors.New("cancellation note should be empty, as the current watering plan is not canceled")
	}

	params := sqlc.UpdateWateringPlanParams{
		ID:                     entity.ID,
		Date:                   date,
		Description:            entity.Description,
		Distance:               entity.Distance,
		TotalWaterRequired:     entity.TotalWaterRequired,
		Status:                 sqlc.WateringPlanStatus(entity.Status),
		CancellationNote:       entity.CancellationNote,
		GpxUrl:                 &entity.GpxURL,
		Duration:               float64(entity.Duration) / float64(time.Second),
		RefillCount:            entity.RefillCount,
		Provider:               &entity.Provider,
		AdditionalInformations: additionalInfo,
	}

	if err := w.store.DeleteAllVehiclesFromWateringPlan(ctx, entity.ID); err != nil {
		return err
	}

	if err := w.setLinkedVehicles(ctx, entity, entity.ID); err != nil {
		return err
	}

	if err := w.store.DeleteAllTreeClusterFromWateringPlan(ctx, entity.ID); err != nil {
		return err
	}

	if err := w.setLinkedTreeClusters(ctx, entity, entity.ID); err != nil {
		return err
	}

	if err := w.updateConsumedWaterValues(ctx, entity); err != nil {
		return err
	}

	if err := w.store.DeleteAllUsersFromWateringPlan(ctx, entity.ID); err != nil {
		return err
	}

	if err := w.setLinkedUsers(ctx, entity, entity.ID); err != nil {
		return err
	}

	return w.store.UpdateWateringPlan(ctx, &params)
}

// This function updates the consumed water values for each tree cluster in a finished watering plan.
// To save the consumed water values, the watering plan must be »finished«
func (w *WateringPlanRepository) updateConsumedWaterValues(ctx context.Context, entity *entities.WateringPlan) error {
	if entity.Status != entities.WateringPlanStatusFinished || len(entity.Evaluation) == 0 {
		return nil
	}

	for _, value := range entity.Evaluation {
		if err := w.store.UpdateTreeClusterWateringPlan(ctx, &sqlc.UpdateTreeClusterWateringPlanParams{
			WateringPlanID: entity.ID,
			TreeClusterID:  value.TreeClusterID,
			ConsumedWater:  *value.ConsumedWater,
		}); err != nil {
			return err
		}
	}

	return nil
}
