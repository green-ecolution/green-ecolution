package vehicle

import (
	"context"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper"

	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	store "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
)

var _ storage.VehicleRepository = (*VehicleRepository)(nil)

type VehicleRepository struct {
	store *store.Store
	VehicleRepositoryMappers
}

type VehicleRepositoryMappers struct {
	mapper mapper.InternalVehicleRepoMapper
}

func NewVehicleRepositoryMappers(vMapper mapper.InternalVehicleRepoMapper) VehicleRepositoryMappers {
	return VehicleRepositoryMappers{
		mapper: vMapper,
	}
}

func NewVehicleRepository(s *store.Store, mappers VehicleRepositoryMappers) *VehicleRepository {
	return &VehicleRepository{
		store:                    s,
		VehicleRepositoryMappers: mappers,
	}
}

func (r *VehicleRepository) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	_, err := r.store.DeleteVehicle(ctx, id)
	if err != nil {
		log.Error("failed to delete vehicle entity in db", "error", err, "vehicle_id", id)
		return err
	}

	log.Debug("vehicle entity deleted successfully in db", "vehicle_id", id)
	return nil
}

func (r *VehicleRepository) Archive(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	now := time.Now()
	_, err := r.store.ArchiveVehicle(ctx, &sqlc.ArchiveVehicleParams{
		ID:         id,
		ArchivedAt: &now,
	})
	if err != nil {
		log.Error("failed to archive vehicle entity in db", "error", err, "vehicle_id", id)
		return err
	}

	log.Debug("vehicle entity archived successfully in db", "vehicle_id", id)
	return nil
}
