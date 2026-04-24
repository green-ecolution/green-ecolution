package cluster

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type TreeClusterRepository interface {
	GetAll(ctx context.Context, query TreeClusterQuery) ([]*TreeCluster, int64, error)
	GetCount(ctx context.Context, query TreeClusterQuery) (int64, error)
	GetByID(ctx context.Context, id int32) (*TreeCluster, error)
	GetByIDs(ctx context.Context, ids []int32) ([]*TreeCluster, error)
	Create(ctx context.Context, entity *TreeCluster) (*TreeCluster, error)
	Update(ctx context.Context, id int32, entity *TreeCluster) error
	Delete(ctx context.Context, id int32) error
	GetAllRegionsWithWateringPlanCount(ctx context.Context) ([]*evaluation.RegionEvaluation, error)

	Archive(ctx context.Context, id int32) error
	LinkTreesToCluster(ctx context.Context, treeClusterID int32, treeIDs []int32) error
	GetCenterPoint(ctx context.Context, id int32) (*shared.Coordinate, error)
	GetAllLatestSensorDataByClusterID(ctx context.Context, tcID int32) ([]*sensor.SensorData, error)
}
