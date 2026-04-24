package wateringplan

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/watering"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

func (w *WateringPlanRepository) GetAll(ctx context.Context, query shared.Query) ([]*watering.WateringPlan, int64, error) {
	log := logger.GetLogger(ctx)
	page, limit, err := pagination.GetValues(ctx)
	if err != nil {
		return nil, 0, w.store.MapError(err, sqlc.WateringPlan{})
	}

	totalCount, err := w.GetCount(ctx, query)
	if err != nil {
		return nil, 0, w.store.MapError(err, sqlc.WateringPlan{})
	}

	if totalCount == 0 {
		return []*watering.WateringPlan{}, 0, nil
	}

	if limit == -1 {
		limit = int32(totalCount)
		page = 1
	}

	rows, err := w.store.GetAllWateringPlans(ctx, &sqlc.GetAllWateringPlansParams{
		Provider: query.Provider,
		Limit:    limit,
		Offset:   (page - 1) * limit,
	})

	if err != nil {
		log.Debug("failed to get watering plan entities in db", "error", err)
		return nil, 0, w.store.MapError(err, sqlc.WateringPlan{})
	}

	data, err := w.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, 0, err
	}

	for _, wp := range data {
		if err := w.mapFields(ctx, wp); err != nil {
			return nil, 0, err
		}
	}

	return data, totalCount, nil
}

func (w *WateringPlanRepository) GetCount(ctx context.Context, query shared.Query) (int64, error) {
	log := logger.GetLogger(ctx)
	totalCount, err := w.store.GetAllWateringPlansCount(ctx, query.Provider)
	if err != nil {
		log.Debug("failed to get total watering plan count in db", "error", err)
		return 0, err
	}

	return totalCount, nil
}

func (w *WateringPlanRepository) GetByID(ctx context.Context, id int32) (*watering.WateringPlan, error) {
	log := logger.GetLogger(ctx)
	row, err := w.store.GetWateringPlanByID(ctx, id)
	if err != nil {
		log.Debug("failed to get watering plan entity by id in db", "error", err, "watering_plan_id", id)
		return nil, w.store.MapError(err, sqlc.WateringPlan{})
	}

	wp, err := w.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to map entity", "error", err)
		return nil, err
	}

	if err := w.mapFields(ctx, wp); err != nil {
		return nil, err
	}

	return wp, nil
}

func (w *WateringPlanRepository) getLinkedVehicleIDByIDAndType(ctx context.Context, id int32, vehicleType string) (*int32, error) {
	log := logger.GetLogger(ctx)
	row, err := w.store.GetVehicleByWateringPlanID(ctx, &sqlc.GetVehicleByWateringPlanIDParams{
		WateringPlanID: id,
		Type:           sqlc.VehicleType(vehicleType),
	})

	if err != nil {
		log.Debug("failed to get linked vehicle entity by id and vehicle type", "error", err, "watering_plan_id", id, "vehicle_type", vehicleType)
		return nil, err
	}

	return &row.ID, nil
}

func (w *WateringPlanRepository) getLinkedTreeClusterIDsByID(ctx context.Context, id int32) ([]int32, error) {
	log := logger.GetLogger(ctx)
	rows, err := w.store.GetTreeClustersByWateringPlanID(ctx, id)
	if err != nil {
		log.Debug("failed to get linked tree cluster entities by watering plan id", "error", err, "watering_plan_id", id)
		return nil, err
	}

	ids := make([]int32, 0, len(rows))
	for _, row := range rows {
		ids = append(ids, row.ID)
	}

	return ids, nil
}

func (w *WateringPlanRepository) getEvaluationValues(ctx context.Context, id int32) ([]*watering.EvaluationValue, error) {
	log := logger.GetLogger(ctx)
	rows, err := w.store.GetAllTreeClusterWateringPlanByID(ctx, id)
	if err != nil {
		log.Debug("failed to get evaluation value entities", "error", err, "watering_plan_id", id)
		return nil, err
	}

	return w.mapper.EvaluationFromSqlList(rows), nil
}

func (w *WateringPlanRepository) getLinkedUsersByID(ctx context.Context, id int32) ([]*uuid.UUID, error) {
	log := logger.GetLogger(ctx)
	UUIDs, err := w.store.GetUsersByWateringPlanID(ctx, id)
	if err != nil {
		log.Error("failed to get linked user entities by watering plan id", "error", err, "watering_plan_id", id)
		return nil, err
	}

	var userUUIDs []*uuid.UUID
	for _, UUID := range UUIDs {
		userUUIDs = append(userUUIDs, &UUID)
	}

	return userUUIDs, nil
}

func (w *WateringPlanRepository) mapFields(ctx context.Context, wp *watering.WateringPlan) error {
	log := logger.GetLogger(ctx)
	var err error

	wp.TreeClusterIDs, err = w.getLinkedTreeClusterIDsByID(ctx, wp.ID)
	if err != nil {
		log.Debug("failed to get linked tree cluster by watering plan id", "error", err, "watering_plan_id", wp.ID)
		return w.store.MapError(err, sqlc.WateringPlan{})
	}

	wp.TransporterID, err = w.getLinkedVehicleIDByIDAndType(ctx, wp.ID, "transporter")
	if err != nil {
		log.Debug("failed to get linked transporter by watering plan id", "error", err, "watering_plan_id", wp.ID)
		return w.store.MapError(err, sqlc.WateringPlan{})
	}

	wp.TrailerID, err = w.getLinkedVehicleIDByIDAndType(ctx, wp.ID, "trailer")
	if err != nil {
		if !errors.Is(err, pgx.ErrNoRows) {
			log.Debug("failed to get linked trailer by watering plan id", "error", err, "watering_plan_id", wp.ID)
			return w.store.MapError(err, sqlc.WateringPlan{})
		}
		wp.TrailerID = nil
	}

	wp.UserIDs, err = w.getLinkedUsersByID(ctx, wp.ID)
	if err != nil {
		log.Debug("failed to get linked users by watering plan id", "error", err, "watering_plan_id", wp.ID)
		return w.store.MapError(err, sqlc.WateringPlan{})
	}

	// Only load evaluation values if the watering plan is set to »finished«
	if wp.Status == watering.WateringPlanStatusFinished {
		wp.Evaluation, err = w.getEvaluationValues(ctx, wp.ID)
		if err != nil {
			log.Debug("failed to get evaluation values by watering plan id", "error", err, "watering_plan_id", wp.ID)
			return w.store.MapError(err, sqlc.WateringPlan{})
		}
	} else {
		wp.Evaluation = []*watering.EvaluationValue{}
	}

	return nil
}
