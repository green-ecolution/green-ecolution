package tree

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type TreeRepository interface {
	GetAll(ctx context.Context, query TreeQuery) ([]*Tree, int64, error)
	GetCount(ctx context.Context, query TreeQuery) (int64, error)
	GetByID(ctx context.Context, id int32) (*Tree, error)
	Create(ctx context.Context, entity *Tree) (*Tree, error)
	Update(ctx context.Context, id int32, entity *Tree) (*Tree, error)
	Delete(ctx context.Context, id int32) error

	FindNearestTrees(ctx context.Context, coord shared.Coordinate, radiusMeters float64, limit int32) ([]*TreeWithDistance, error)
	GetDistinctPlantingYears(ctx context.Context) ([]int32, error)

	UnlinkTreeClusterID(ctx context.Context, treeClusterID int32) error
	UnlinkSensorID(ctx context.Context, sensorID sensor.SensorID) error
}
