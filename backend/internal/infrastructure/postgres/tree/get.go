package tree

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
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

	rows, err := r.store.GetAllTrees(ctx, &sqlc.GetAllTreesParams{
		WateringStatus: wateringStatuses,
		Provider:       query.Provider,
		Years:          query.PlantingYears,
		HasCluster:     query.HasCluster,
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

	totalCount, err := r.store.GetAllTreesCount(ctx, &sqlc.GetAllTreesCountParams{
		WateringStatus: wateringStatuses,
		Provider:       query.Provider,
		Years:          query.PlantingYears,
		HasCluster:     query.HasCluster,
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

func (r *TreeRepository) GetBySensorID(ctx context.Context, id sensor.SensorID) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	idStr := id.String()
	row, err := r.store.GetTreeBySensorID(ctx, &idStr)
	if err != nil {
		log.Debug("failed to get tree by sensor id in db", "error", err, "sensor_id", id)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return t, nil
}

func (r *TreeRepository) GetBySensorIDs(ctx context.Context, ids ...sensor.SensorID) ([]*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	idStrs := make([]string, len(ids))
	for i, id := range ids {
		idStrs[i] = id.String()
	}
	rows, err := r.store.GetTreesBySensorIDs(ctx, idStrs)
	if err != nil {
		log.Debug("failed to get trees by multiple sensor ids in db", "error", err, "sensor_ids", ids)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return t, nil
}

func (r *TreeRepository) GetTreesByIDs(ctx context.Context, ids []int32) ([]*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	rows, err := r.store.GetTreesByIDs(ctx, ids)
	if err != nil {
		log.Debug("failed to get trees by ids in db", "error", err, "tree_ids", ids)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return t, nil
}

func (r *TreeRepository) GetByTreeClusterID(ctx context.Context, id int32) ([]*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	rows, err := r.store.GetTreesByTreeClusterID(ctx, &id)
	if err != nil {
		log.Debug("failed to get tree by cluster id in db", "error", err, "cluster_id", id)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return t, nil
}

func (r *TreeRepository) GetByCoordinates(ctx context.Context, coord shared.Coordinate) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	params := sqlc.GetTreeByCoordinatesParams{
		Latitude:  coord.Latitude(),
		Longitude: coord.Longitude(),
	}
	row, err := r.store.GetTreeByCoordinates(ctx, &params)
	if err != nil {
		log.Debug("failed to get tree by coordinates in db", "error", err, "latitude", coord.Latitude(), "longitude", coord.Longitude())
		return nil, r.store.MapError(err, sqlc.Tree{})
	}
	t, err := r.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}
	return t, nil
}

func (r *TreeRepository) GetSensorByTreeID(ctx context.Context, treeID int32) (*sensor.Sensor, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetSensorByTreeID(ctx, treeID)
	if err != nil {
		log.Debug("failed to get sensor by tree id", "error", err, "tree_id", treeID)
		return nil, r.store.MapError(err, sqlc.Sensor{})
	}

	data, err := r.sMapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	if err := r.store.MapSensorFields(ctx, data); err != nil {
		log.Debug("failed to convert sensor data", "error", err)
		return nil, r.store.MapError(err, sqlc.Sensor{})
	}

	return data, nil
}

func (r *TreeRepository) getTreeClusterByTreeID(ctx context.Context, treeID int32) (*cluster.TreeCluster, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetTreeClusterByTreeID(ctx, treeID)
	if err != nil {
		return nil, err
	}

	tc, err := r.tcMapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return tc, nil
}

func (r *TreeRepository) FindNearestTree(ctx context.Context, coord shared.Coordinate) (*treeDomain.Tree, error) {
	log := logger.GetLogger(ctx)
	params := &sqlc.FindNearestTreeParams{
		StMakepoint:   coord.Latitude(),
		StMakepoint_2: coord.Longitude(),
	}

	nearestTree, err := r.store.FindNearestTree(ctx, params)
	if err != nil {
		log.Debug("failed to find nearest tree on given coordinates", "error", err, "latitude", coord.Latitude(), "longitude", coord.Longitude())
		return nil, err
	}

	t, err := r.mapper.FromSql(nearestTree)
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
