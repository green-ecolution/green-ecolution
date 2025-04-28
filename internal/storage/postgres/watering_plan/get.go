package wateringplan

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/utils/pagination"
	"github.com/jackc/pgx/v5"
)

func (w *WateringPlanRepository) GetAll(ctx context.Context, query entities.Query) ([]*entities.WateringPlan, int64, error) {
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
		return []*entities.WateringPlan{}, 0, nil
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

func (w *WateringPlanRepository) GetCount(ctx context.Context, query entities.Query) (int64, error) {
	log := logger.GetLogger(ctx)
	totalCount, err := w.store.GetAllWateringPlansCount(ctx, query.Provider)
	if err != nil {
		log.Debug("failed to get total watering plan count in db", "error", err)
		return 0, err
	}

	return totalCount, nil
}

func (w *WateringPlanRepository) GetByID(ctx context.Context, id int32) (*entities.WateringPlan, error) {
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

func (w *WateringPlanRepository) GetLinkedVehicleByIDAndType(ctx context.Context, id int32, vehicleType entities.VehicleType) (*entities.Vehicle, error) {
	log := logger.GetLogger(ctx)
	row, err := w.store.GetVehicleByWateringPlanID(ctx, &sqlc.GetVehicleByWateringPlanIDParams{
		WateringPlanID: id,
		Type:           sqlc.VehicleType(vehicleType),
	})

	if err != nil {
		log.Debug("failed to get linked vehicle entity by id and vehicle type", "error", err, "watering_plan_id", id, "vehicle_type", vehicleType)
		return nil, err
	}

	vehicle, err := w.vehicleMapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return vehicle, nil
}

func (w *WateringPlanRepository) GetLinkedTreeClustersByID(ctx context.Context, id int32) ([]*entities.TreeCluster, error) {
	log := logger.GetLogger(ctx)
	rows, err := w.store.GetTreeClustersByWateringPlanID(ctx, id)
	if err != nil {
		log.Debug("failed to get linked tree cluster entities by watering plan id", "error", err, "watering_plan_id", id)
		return nil, err
	}

	tc, err := w.clusterMapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	for _, cluster := range tc {
		if err := w.store.MapClusterFields(ctx, cluster); err != nil {
			return nil, err
		}
	}

	return tc, nil
}

func (w *WateringPlanRepository) GetEvaluationValues(ctx context.Context, id int32) ([]*entities.EvaluationValue, error) {
	log := logger.GetLogger(ctx)
	rows, err := w.store.GetAllTreeClusterWateringPlanByID(ctx, id)
	if err != nil {
		log.Debug("failed to get evaluation value entities", "error", err, "watering_plan_id", id)
		return nil, err
	}

	return w.mapper.EvaluationFromSqlList(rows), nil
}

func (w *WateringPlanRepository) GetLinkedUsersByID(ctx context.Context, id int32) ([]*uuid.UUID, error) {
	log := logger.GetLogger(ctx)
	pgUUIDS, err := w.store.GetUsersByWateringPlanID(ctx, id)
	if err != nil {
		log.Error("failed to get linked user entities by watering plan id", "error", err, "watering_plan_id", id)
		return nil, err
	}

	// Convert pgtype.UUID to uuid.UUID
	var userUUIDs []*uuid.UUID
	for _, pgUUID := range pgUUIDS {
		if pgUUID.Valid {
			uuidVal := uuid.UUID(pgUUID.Bytes)
			userUUIDs = append(userUUIDs, &uuidVal)
		}
	}

	return userUUIDs, nil
}

func (w *WateringPlanRepository) GetTotalConsumedWater(ctx context.Context) (int64, error) {
	log := logger.GetLogger(ctx)
	totalConsumedWater, err := w.store.GetTotalConsumedWater(ctx)
	if err != nil {
		log.Debug("failed to get total consumed water value in db", "error", err)
		return 0, err
	}

	return totalConsumedWater, nil
}

func (w *WateringPlanRepository) GetAllUserCount(ctx context.Context) (int64, error) {
	log := logger.GetLogger(ctx)
	userCount, err := w.store.GetAllUserWateringPlanCount(ctx)
	if err != nil {
		log.Debug("failed to get count of all user linked to watering plans in db", "error", err)
		return 0, err
	}

	return userCount, nil
}

func (w *WateringPlanRepository) mapFields(ctx context.Context, wp *entities.WateringPlan) error {
	log := logger.GetLogger(ctx)
	var err error

	wp.TreeClusters, err = w.GetLinkedTreeClustersByID(ctx, wp.ID)
	if err != nil {
		log.Debug("failed to get linked tree cluster by watering plan id", "error", err, "watering_plan_id", wp.ID)
		return w.store.MapError(err, sqlc.WateringPlan{})
	}

	wp.Transporter, err = w.GetLinkedVehicleByIDAndType(ctx, wp.ID, entities.VehicleTypeTransporter)
	if err != nil {
		log.Debug("failed to get linked transporter by watering plan id", "error", err, "watering_plan_id", wp.ID)
		return w.store.MapError(err, sqlc.WateringPlan{})
	}

	wp.Trailer, err = w.GetLinkedVehicleByIDAndType(ctx, wp.ID, entities.VehicleTypeTrailer)
	if err != nil {
		if !errors.Is(err, pgx.ErrNoRows) {
			log.Debug("failed to get linked trailer by watering plan id", "error", err, "watering_plan_id", wp.ID)
			return w.store.MapError(err, sqlc.WateringPlan{})
		}
		wp.Trailer = nil
	}

	wp.UserIDs, err = w.GetLinkedUsersByID(ctx, wp.ID)
	if err != nil {
		log.Debug("failed to get linked users by watering plan id", "error", err, "watering_plan_id", wp.ID)
		return w.store.MapError(err, sqlc.WateringPlan{})
	}

	// Only load evaluation values if the watering plan is set to »finished«
	if wp.Status == entities.WateringPlanStatusFinished {
		wp.Evaluation, err = w.GetEvaluationValues(ctx, wp.ID)
		if err != nil {
			log.Debug("failed to get evaluation values by watering plan id", "error", err, "watering_plan_id", wp.ID)
			return w.store.MapError(err, sqlc.WateringPlan{})
		}
	} else {
		wp.Evaluation = []*entities.EvaluationValue{}
	}

	return nil
}
