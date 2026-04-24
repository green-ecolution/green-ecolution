package tree

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	imgMapper "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

type TreeRepository struct {
	store *store.Store
	TreeMappers
}

type TreeMappers struct {
	mapper   imgMapper.InternalTreeRepoMapper
	sMapper  imgMapper.InternalSensorRepoMapper
	tcMapper imgMapper.InternalTreeClusterRepoMapper
}

func NewTreeRepositoryMappers(
	tMapper imgMapper.InternalTreeRepoMapper,
	sMapper imgMapper.InternalSensorRepoMapper,
	tcMapper imgMapper.InternalTreeClusterRepoMapper,
) TreeMappers {
	return TreeMappers{
		mapper:   tMapper,
		sMapper:  sMapper,
		tcMapper: tcMapper,
	}
}

var _ tree.TreeRepository = (*TreeRepository)(nil)

func NewTreeRepository(s *store.Store, mappers TreeMappers) *TreeRepository {
	return &TreeRepository{
		store:       s,
		TreeMappers: mappers,
	}
}

func (r *TreeRepository) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)

	_, err := r.store.DeleteTree(ctx, id)
	if err != nil {
		log.Debug("failed to delete tree in db", "error", err, "tree_id", id)
		return err
	}

	log.Debug("tree entity deleted successfully in db", "tree_id", id)
	return nil
}

func (r *TreeRepository) UnlinkTreeClusterID(ctx context.Context, treeClusterID int32) error {
	log := logger.GetLogger(ctx)

	_, err := r.store.GetTreeClusterByID(ctx, treeClusterID)
	if err != nil {
		return err
	}
	unlinkTreeIDs, err := r.store.UnlinkTreeClusterID(ctx, &treeClusterID)
	if err != nil {
		log.Error("failed to unlink tree cluster from trees", "error", err, "cluster_id", treeClusterID)
	}

	log.Info("unlink trees from following tree cluster", "cluster_id", treeClusterID, "unlinked_trees", unlinkTreeIDs)

	return nil
}

func (r *TreeRepository) UnlinkSensorID(ctx context.Context, sensorID sensor.SensorID) error {
	if sensorID.String() == "" {
		return errors.New("sensorID cannot be empty")
	}
	idStr := sensorID.String()
	return r.store.UnlinkSensorIDFromTrees(ctx, &idStr)
}

func (r *TreeRepository) GetDistinctPlantingYears(ctx context.Context) ([]int32, error) {
	return r.store.GetDistinctPlantingYears(ctx)
}
