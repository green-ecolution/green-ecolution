package vehicle

import (
	"context"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/utils/pagination"
)

func (r *VehicleRepository) getHelper(
	ctx context.Context,
	totalCountFn func() (int64, error),
	entitiesFn func(page, limit int32) ([]*sqlc.Vehicle, error),
) ([]*entities.Vehicle, int64, error) {
	log := logger.GetLogger(ctx)
	page, limit, err := pagination.GetValues(ctx)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Vehicle{})
	}

	totalCount, err := totalCountFn()
	if err != nil {
		log.Debug("failed to get total vehicle count in db", "error", err)
		return nil, 0, r.store.MapError(err, sqlc.Vehicle{})
	}

	if totalCount == 0 {
		return []*entities.Vehicle{}, 0, nil
	}

	if limit == -1 {
		limit = int32(totalCount)
		page = 1
	}

	rows, err := entitiesFn(page, limit)
	if err != nil {
		log.Debug("failed to get vehicle entities in db", "error", err)
		return nil, 0, r.store.MapError(err, sqlc.Vehicle{})
	}

	return r.mapFromList(ctx, rows, totalCount)
}

func (r *VehicleRepository) GetAll(ctx context.Context, query entities.Query) ([]*entities.Vehicle, int64, error) {
	return r.getHelper(
		ctx,
		func() (int64, error) {
			return r.store.GetAllVehiclesCount(ctx, query.Provider)
		},
		func(page, limit int32) ([]*sqlc.Vehicle, error) {
			return r.store.GetAllVehicles(ctx, &sqlc.GetAllVehiclesParams{
				Provider: query.Provider,
				Limit:    limit,
				Offset:   (page - 1) * limit,
			})
		},
	)
}

func (r *VehicleRepository) GetAllWithArchived(ctx context.Context, provider string) ([]*entities.Vehicle, int64, error) {
	return r.getHelper(
		ctx,
		func() (int64, error) {
			return r.store.GetAllVehiclesWithArchivedCount(ctx, provider)
		},
		func(page, limit int32) ([]*sqlc.Vehicle, error) {
			return r.store.GetAllVehiclesWithArchived(ctx, &sqlc.GetAllVehiclesWithArchivedParams{
				Provider: provider,
				Limit:    limit,
				Offset:   (page - 1) * limit,
			})
		},
	)
}

func (r *VehicleRepository) GetAllByTypeWithArchived(ctx context.Context, provider string, vehicleType entities.VehicleType) ([]*entities.Vehicle, int64, error) {
	return r.getHelper(
		ctx,
		func() (int64, error) {
			return r.store.GetAllVehiclesByTypeWithArchivedCount(ctx, &sqlc.GetAllVehiclesByTypeWithArchivedCountParams{
				Type:     sqlc.VehicleType(vehicleType),
				Provider: provider})
		},
		func(page, limit int32) ([]*sqlc.Vehicle, error) {
			return r.store.GetAllVehiclesByTypeWithArchived(ctx, &sqlc.GetAllVehiclesByTypeWithArchivedParams{
				Type:     sqlc.VehicleType(vehicleType),
				Provider: provider,
				Limit:    limit,
				Offset:   (page - 1) * limit,
			})
		},
	)
}

func (r *VehicleRepository) GetAllByType(ctx context.Context, provider string, vehicleType entities.VehicleType) ([]*entities.Vehicle, int64, error) {
	return r.getHelper(
		ctx,
		func() (int64, error) {
			return r.store.GetAllVehiclesByTypeCount(ctx, &sqlc.GetAllVehiclesByTypeCountParams{
				Type:     sqlc.VehicleType(vehicleType),
				Provider: provider,
			})
		},
		func(page, limit int32) ([]*sqlc.Vehicle, error) {
			return r.store.GetAllVehiclesByType(ctx, &sqlc.GetAllVehiclesByTypeParams{
				Type:     sqlc.VehicleType(vehicleType),
				Provider: provider,
				Limit:    limit,
				Offset:   (page - 1) * limit,
			})
		},
	)
}

func (r *VehicleRepository) GetAllArchived(ctx context.Context) ([]*entities.Vehicle, error) {
	log := logger.GetLogger(ctx)
	rows, err := r.store.GetAllArchivedVehicles(ctx)
	if err != nil {
		log.Debug("failed to get archived vehicle entities", "error", err)
		return nil, r.store.MapError(err, sqlc.Vehicle{})
	}

	return r.mapper.FromSqlList(rows)
}

func (r *VehicleRepository) GetAllWithWateringPlanCount(ctx context.Context) ([]*entities.VehicleEvaluation, error) {
	log := logger.GetLogger(ctx)
	rows, err := r.store.GetAllVehiclesWithWateringPlanCount(ctx)
	if err != nil {
		log.Debug("failed to get vehicles with watering plan count", "error", err)
		return nil, r.store.MapError(err, sqlc.GetAllVehiclesWithWateringPlanCountRow{})
	}

	return r.mapper.FromSqlListVehicleWithCount(rows)
}

func (r *VehicleRepository) GetByID(ctx context.Context, id int32) (*entities.Vehicle, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetVehicleByID(ctx, id)
	if err != nil {
		log.Debug("failed to get vehicle entity by provided id", "error", err, "vehicle_id", id)
		return nil, r.store.MapError(err, sqlc.Vehicle{})
	}

	return r.mapFromRow(ctx, row)
}

func (r *VehicleRepository) GetByPlate(ctx context.Context, plate string) (*entities.Vehicle, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetVehicleByPlate(ctx, plate)
	if err != nil {
		log.Debug("failed to get vehicle entity by given plate", "error", err, "vehicle_plate", plate)
		return nil, r.store.MapError(err, sqlc.Vehicle{})
	}

	return r.mapFromRow(ctx, row)
}

func (r *VehicleRepository) mapFromRow(ctx context.Context, rows *sqlc.Vehicle) (*entities.Vehicle, error) {
	log := logger.GetLogger(ctx)
	vehicles, err := r.mapper.FromSql(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return vehicles, nil
}

func (r *VehicleRepository) mapFromList(ctx context.Context, rows []*sqlc.Vehicle, totalCount int64) ([]*entities.Vehicle, int64, error) {
	log := logger.GetLogger(ctx)
	vehicles, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, 0, err
	}

	return vehicles, totalCount, nil
}
