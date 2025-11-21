package tree

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
	"github.com/jackc/pgx/v5"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/pkg/errors"
)

func (r *TreeRepository) GetAll(ctx context.Context, query entities.TreeQuery) ([]*entities.Tree, int64, error) {
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
		return []*entities.Tree{}, 0, nil
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

	for _, tree := range t {
		if err := r.mapFields(ctx, tree); err != nil {
			return nil, 0, err
		}
	}

	return t, totalCount, nil
}

func (r *TreeRepository) GetCount(ctx context.Context, query entities.TreeQuery) (int64, error) {
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

func (r *TreeRepository) GetByID(ctx context.Context, id int32) (*entities.Tree, error) {
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

	if err := r.mapFields(ctx, t); err != nil {
		return nil, err
	}

	return t, nil
}

func (r *TreeRepository) GetBySensorID(ctx context.Context, id string) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetTreeBySensorID(ctx, &id)
	if err != nil {
		log.Debug("failed to get tree by sensor id in db", "error", err, "sensor_id", id)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	if err := r.mapFields(ctx, t); err != nil {
		return nil, err
	}

	return t, nil
}

func (r *TreeRepository) GetBySensorIDs(ctx context.Context, ids ...string) ([]*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	rows, err := r.store.GetTreesBySensorIDs(ctx, ids)
	if err != nil {
		log.Debug("failed to get trees by multiple sensor ids in db", "error", err, "sensor_ids", ids)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}

	t, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	for _, tree := range t {
		if err := r.mapFields(ctx, tree); err != nil {
			return nil, err
		}
	}

	return t, nil
}

func (r *TreeRepository) GetTreesByIDs(ctx context.Context, ids []int32) ([]*entities.Tree, error) {
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

	for _, tree := range t {
		if err := r.mapFields(ctx, tree); err != nil {
			return nil, err
		}
	}

	return t, nil
}

func (r *TreeRepository) GetByTreeClusterID(ctx context.Context, id int32) ([]*entities.Tree, error) {
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

	for _, tree := range t {
		if err := r.mapFields(ctx, tree); err != nil {
			return nil, err
		}
	}

	return t, nil
}

func (r *TreeRepository) GetByCoordinates(ctx context.Context, latitude, longitude float64) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	params := sqlc.GetTreeByCoordinatesParams{
		Latitude:  latitude,
		Longitude: longitude,
	}
	row, err := r.store.GetTreeByCoordinates(ctx, &params)
	if err != nil {
		log.Debug("failed to get tree by coordinates in db", "error", err, "latitude", latitude, "longitude", longitude)
		return nil, r.store.MapError(err, sqlc.Tree{})
	}
	tree, err := r.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}
	if err := r.mapFields(ctx, tree); err != nil {
		return nil, err
	}
	return tree, nil
}

func (r *TreeRepository) GetSensorByTreeID(ctx context.Context, treeID int32) (*entities.Sensor, error) {
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

func (r *TreeRepository) getTreeClusterByTreeID(ctx context.Context, treeID int32) (*entities.TreeCluster, error) {
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

// Map sensor and tree cluster entity to domain tree
func (r *TreeRepository) mapFields(ctx context.Context, t *entities.Tree) error {
	if err := mapSensor(ctx, r, t); err != nil {
		return err
	}

	if err := mapTreeCluster(ctx, r, t); err != nil {
		return err
	}

	return nil
}

func mapSensor(ctx context.Context, r *TreeRepository, t *entities.Tree) error {
	sensor, err := r.GetSensorByTreeID(ctx, t.ID)
	if err != nil {
		var entityNotFoundErr storage.ErrEntityNotFound
		if errors.As(err, &entityNotFoundErr) {
			// If sensor is not found, set sensor to nil
			t.Sensor = nil
			return nil
		}
		return err
	}
	t.Sensor = sensor
	return nil
}

func mapTreeCluster(ctx context.Context, r *TreeRepository, t *entities.Tree) error {
	treeCluster, err := r.getTreeClusterByTreeID(ctx, t.ID)
	if err != nil && !errors.Is(err, pgx.ErrNoRows) {
		return err
	}
	t.TreeCluster = treeCluster
	return nil
}

func (r *TreeRepository) FindNearestTree(ctx context.Context, latitude, longitude float64) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	params := &sqlc.FindNearestTreeParams{
		StMakepoint:   latitude,
		StMakepoint_2: longitude,
	}

	nearestTree, err := r.store.FindNearestTree(ctx, params)
	if err != nil {
		log.Debug("failed to find nearest tree on given coordinates", "error", err, "latitude", latitude, "longitude", longitude)
		return nil, err
	}

	tree, err := r.mapper.FromSql(nearestTree)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	if err := r.mapFields(ctx, tree); err != nil {
		return nil, err
	}
	return tree, nil
}
