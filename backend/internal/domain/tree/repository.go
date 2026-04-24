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
	Create(ctx context.Context, fn func(tree *Tree, repo TreeRepository) (bool, error)) (*Tree, error)
	Update(ctx context.Context, id int32, updateFn func(tree *Tree, repo TreeRepository) (bool, error)) (*Tree, error)
	Delete(ctx context.Context, id int32) error

	GetByTreeClusterID(ctx context.Context, id int32) ([]*Tree, error)
	GetSensorByTreeID(ctx context.Context, id int32) (*sensor.Sensor, error)
	GetTreesByIDs(ctx context.Context, ids []int32) ([]*Tree, error)
	GetByCoordinates(ctx context.Context, coord shared.Coordinate) (*Tree, error)
	GetBySensorID(ctx context.Context, id sensor.SensorID) (*Tree, error)
	GetBySensorIDs(ctx context.Context, ids ...sensor.SensorID) ([]*Tree, error)

	UnlinkTreeClusterID(ctx context.Context, treeClusterID int32) error
	UnlinkSensorID(ctx context.Context, sensorID sensor.SensorID) error
	FindNearestTree(ctx context.Context, coord shared.Coordinate) (*Tree, error)
	FindNearestTrees(ctx context.Context, coord shared.Coordinate, radiusMeters float64, limit int32) ([]*TreeWithDistance, error)
	GetDistinctPlantingYears(ctx context.Context) ([]int32, error)
}
