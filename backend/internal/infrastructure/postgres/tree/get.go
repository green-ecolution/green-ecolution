package tree

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	treeDomain "github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

func (r *TreeRepository) GetAll(ctx context.Context, query treeDomain.TreeQuery) ([]*treeDomain.Tree, int64, error) {
	log := logger.GetLogger(ctx)
	page, limit, err := pagination.GetValues(ctx)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Tree{})
	}

	totalCount, err := r.GetCount(ctx, query)

	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Tree{})
	}

	if totalCount == 0 {
		return []*treeDomain.Tree{}, 0, nil
	}

	if limit == -1 {
		limit = int32(totalCount)
		page = 1
	}

	var wateringStatuses []string
	for _, ws := range query.WateringStatuses {
		wateringStatuses = append(wateringStatuses, string(ws))
	}

	var sensorID *string
	if query.SensorID != nil {
		s := query.SensorID.String()
		sensorID = &s
	}

	rows, err := r.store.GetAllTrees(ctx, &sqlc.GetAllTreesParams{
		WateringStatus: wateringStatuses,
		Provider:       query.Provider,
		Years:          query.PlantingYears,
		HasCluster:     query.HasCluster,
		TreeClusterID:  query.TreeClusterID,
		SensorID:       sensorID,
		Ids:            query.IDs,
		Limit:          limit,
		Offset:         (page - 1) * limit,
	})

	if err != nil {
		log.Debug("failed to get trees in db", "error", err)
		return nil, 0, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, 0, err
	}

	return t, totalCount, nil
}

func (r *TreeRepository) GetCount(ctx context.Context, query treeDomain.TreeQuery) (int64, error) {
	log := logger.GetLogger(ctx)

	var wateringStatuses []string
	for _, ws := range query.WateringStatuses {
		wateringStatuses = append(wateringStatuses, string(ws))
	}

	var sensorID *string
	if query.SensorID != nil {
		s := query.SensorID.String()
		sensorID = &s
	}

	totalCount, err := r.store.GetAllTreesCount(ctx, &sqlc.GetAllTreesCountParams{
		WateringStatus: wateringStatuses,
		Provider:       query.Provider,
		Years:          query.PlantingYears,
		HasCluster:     query.HasCluster,
		TreeClusterID:  query.TreeClusterID,
		SensorID:       sensorID,
		Ids:            query.IDs,
	})

	if err != nil {
		log.Debug("failed to get total trees count in db", "error", err)
		return 0, err
	}

	return totalCount, nil
}

func (r *TreeRepository) GetByID(ctx context.Context, id int32) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetTreeByID(ctx, id)
	if err != nil {
		log.Debug("failed to get tree by id in db", "error", err, "tree_id", id)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return t, nil
}

func (r *TreeRepository) FindNearestTrees(ctx context.Context, coord shared.Coordinate, radiusMeters float64, limit int32) ([]*treeDomain.TreeWithDistance, error) {
	log := logger.GetLogger(ctx)
	params := &sqlc.FindNearestTreesParams{
		Lat:        coord.Latitude(),
		Lng:        coord.Longitude(),
		Radius:     radiusMeters,
		MaxResults: limit,
	}

	rows, err := r.store.FindNearestTrees(ctx, params)
	if err != nil {
		log.Debug("failed to find nearest trees", "error", err, "lat", coord.Latitude(), "lng", coord.Longitude())
		return nil, err
	}

	results := make([]*treeDomain.TreeWithDistance, 0, len(rows))
	for _, row := range rows {
		t, err := r.mapper.FromSql(&row.Tree)
		if err != nil {
			log.Debug("failed to convert entity", "error", err)
			return nil, err
		}
		results = append(results, &treeDomain.TreeWithDistance{
			Tree:     t,
			Distance: shared.MustNewDistance(row.Distance),
		})
	}
	return results, nil
}
