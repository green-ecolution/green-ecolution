package sensor

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type SensorRepository interface {
	GetAll(ctx context.Context, query shared.Query) ([]*Sensor, int64, error)
	GetCount(ctx context.Context, query shared.Query) (int64, error)
	GetByID(ctx context.Context, id SensorID) (*Sensor, error)
	Create(ctx context.Context, createFn func(*Sensor, SensorRepository) (bool, error)) (*Sensor, error)
	Update(ctx context.Context, id SensorID, updateFn func(*Sensor, SensorRepository) (bool, error)) (*Sensor, error)
	Delete(ctx context.Context, id SensorID) error

	GetAllDataByID(ctx context.Context, id SensorID) ([]*SensorData, error)
	GetLatestSensorDataBySensorID(ctx context.Context, id SensorID) (*SensorData, error)
	InsertSensorData(ctx context.Context, data *SensorData, id SensorID) error
}
