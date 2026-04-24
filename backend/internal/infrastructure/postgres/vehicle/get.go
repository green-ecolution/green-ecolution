package vehicle

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

func (r *VehicleRepository) GetAll(ctx context.Context, query vehicle.VehicleQuery) ([]*vehicle.Vehicle, int64, error) {
	log := logger.GetLogger(ctx)
	page, limit, err := pagination.GetValues(ctx)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Vehicle{})
	}

	totalCount, err := r.GetCount(ctx, query)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Vehicle{})
	}

	if totalCount == 0 {
		return []*vehicle.Vehicle{}, 0, nil
	}

	if limit == -1 {
		limit = int32(totalCount)
		page = 1
	}

	var vehicleType sqlc.NullVehicleType
	if query.Type != "" {
		vehicleType = sqlc.NullVehicleType{
			VehicleType: sqlc.VehicleType(query.Type),
			Valid:       true,
		}
	}

	rows, err := r.store.GetAllVehicles(ctx, &sqlc.GetAllVehiclesParams{
		Provider:     query.Provider,
		VehicleType:  vehicleType,
		OnlyArchived: query.OnlyArchived,
		WithArchived: query.WithArchived,
		Limit:        limit,
		Offset:       (page - 1) * limit,
	})
	if err != nil {
		log.Debug("failed to get vehicle entities in db", "error", err)
		return nil, 0, r.store.MapError(err, sqlc.Vehicle{})
	}

	vehicles, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, 0, err
	}

	return vehicles, totalCount, nil
}

func (r *VehicleRepository) GetCount(ctx context.Context, query vehicle.VehicleQuery) (int64, error) {
	log := logger.GetLogger(ctx)

	var vehicleType sqlc.NullVehicleType
	if query.Type != "" {
		vehicleType = sqlc.NullVehicleType{
			VehicleType: sqlc.VehicleType(query.Type),
			Valid:       true,
		}
	}

	count, err := r.store.GetAllVehiclesCount(ctx, &sqlc.GetAllVehiclesCountParams{
		Provider:     query.Provider,
		VehicleType:  vehicleType,
		OnlyArchived: query.OnlyArchived,
		WithArchived: query.WithArchived,
	})
	if err != nil {
		log.Debug("failed to get total vehicles count in db", "error", err)
		return 0, err
	}
	return count, nil
}

func (r *VehicleRepository) GetByID(ctx context.Context, id int32) (*vehicle.Vehicle, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetVehicleByID(ctx, id)
	if err != nil {
		log.Debug("failed to get vehicle entity by provided id", "error", err, "vehicle_id", id)
		return nil, r.store.MapError(err, sqlc.Vehicle{})
	}

	return r.mapFromRow(ctx, row)
}

func (r *VehicleRepository) GetByPlate(ctx context.Context, plate string) (*vehicle.Vehicle, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetVehicleByPlate(ctx, plate)
	if err != nil {
		log.Debug("failed to get vehicle entity by given plate", "error", err, "vehicle_plate", plate)
		return nil, r.store.MapError(err, sqlc.Vehicle{})
	}

	return r.mapFromRow(ctx, row)
}

func (r *VehicleRepository) mapFromRow(ctx context.Context, rows *sqlc.Vehicle) (*vehicle.Vehicle, error) {
	log := logger.GetLogger(ctx)
	vehicles, err := r.mapper.FromSql(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return vehicles, nil
}
