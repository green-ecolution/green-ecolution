package treecluster

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

func (r *TreeClusterRepository) GetAll(ctx context.Context, filter cluster.TreeClusterQuery) ([]*cluster.TreeCluster, int64, error) {
	log := logger.GetLogger(ctx)

	page, limit, err := pagination.GetValues(ctx)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.TreeCluster{})
	}

	var wateringStatuses []string
	for _, ws := range filter.WateringStatuses {
		wateringStatuses = append(wateringStatuses, string(ws))
	}

	totalCount, err := r.GetCount(ctx, filter)

	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.TreeCluster{})
	}

	if totalCount == 0 {
		return []*cluster.TreeCluster{}, 0, nil
	}

	if limit == -1 {
		limit = int32(totalCount)
		page = 1
	}

	rows, err := r.store.GetAllTreeClusters(ctx, &sqlc.GetAllTreeClustersParams{
		WateringStatus: wateringStatuses,
		Region:         filter.Regions,
		Provider:       filter.Provider,
		Ids:            filter.IDs,
		Limit:          limit,
		Offset:         (page - 1) * limit,
	})

	if err != nil {
		log.Debug("failed to get tree clusters in db")
		return nil, 0, r.store.MapError(err, sqlc.TreeCluster{})
	}

	data, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, 0, err
	}

	for _, tc := range data {
		if err := r.store.MapClusterFields(ctx, tc); err != nil {
			return nil, 0, r.store.MapError(err, sqlc.TreeCluster{})
		}
	}

	return data, totalCount, nil
}

func (r *TreeClusterRepository) GetCount(ctx context.Context, filter cluster.TreeClusterQuery) (int64, error) {
	log := logger.GetLogger(ctx)
	var wateringStatuses []string
	for _, ws := range filter.WateringStatuses {
		wateringStatuses = append(wateringStatuses, string(ws))
	}

	totalCount, err := r.store.GetTreeClustersCount(ctx, &sqlc.GetTreeClustersCountParams{
		WateringStatus: wateringStatuses,
		Region:         filter.Regions,
		Provider:       filter.Provider,
		Ids:            filter.IDs,
	})
	if err != nil {
		log.Debug("failed to get total tree cluster count in db", "error", err)
		return 0, err
	}

	return totalCount, nil
}

func (r *TreeClusterRepository) GetByID(ctx context.Context, id int32) (*cluster.TreeCluster, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetTreeClusterByID(ctx, id)
	if err != nil {
		log.Debug("failed to get tree cluster by id in db", "error", err, "cluster_id", id)
		return nil, r.store.MapError(err, sqlc.TreeCluster{})
	}

	tc, err := r.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	if err := r.store.MapClusterFields(ctx, tc); err != nil {
		return nil, r.store.MapError(err, sqlc.TreeCluster{})
	}

	return tc, nil
}

func (r *TreeClusterRepository) GetCenterPoint(ctx context.Context, tcID int32) (*shared.Coordinate, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.CalculateTreesCentroid(ctx, &tcID)
	if err != nil {
		log.Warn("failed to calculate center point of given cluster", "error", err, "cluster_id", tcID)
		return nil, err
	}

	coord, err := shared.NewCoordinate(row.CenterX, row.CenterY)
	if err != nil {
		return nil, err
	}
	return &coord, nil
}
