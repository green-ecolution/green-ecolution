package sensor

import (
	"context"

	sensorDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

var _ sensorDomain.SensorRepository = (*SensorRepository)(nil)

type SensorRepository struct {
	store *store.Store
	SensorRepositoryMappers
}

type SensorRepositoryMappers struct {
	mapper mapper.InternalSensorRepoMapper
}

func NewSensorRepositoryMappers(sMapper mapper.InternalSensorRepoMapper) SensorRepositoryMappers {
	return SensorRepositoryMappers{
		mapper: sMapper,
	}
}

func NewSensorRepository(s *store.Store, mappers SensorRepositoryMappers) *SensorRepository {
	return &SensorRepository{
		store:                   s,
		SensorRepositoryMappers: mappers,
	}
}

func (r *SensorRepository) Delete(ctx context.Context, id sensorDomain.SensorID) error {
	log := logger.GetLogger(ctx)
	if err := r.store.DeleteSensor(ctx, id.String()); err != nil {
		log.Error("failed to delete sensor entity in db", "error", err, "sensor_id", id.String())
		return err
	}

	log.Debug("sensor entity deleted successfully in db", "sensor_id", id.String())
	return nil
}
