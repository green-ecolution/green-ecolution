package treecluster

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/store"
)

type TreeClusterRepository struct {
	store *store.Store
	TreeClusterMappers
}

type TreeClusterMappers struct {
	mapper       mapper.InternalTreeClusterRepoMapper
	sensorMapper mapper.InternalSensorRepoMapper
	regionMapper mapper.InternalRegionRepoMapper
	treeMapper   mapper.InternalTreeRepoMapper
}

func NewTreeClusterRepositoryMappers(
	tcMapper mapper.InternalTreeClusterRepoMapper,
	sMapper mapper.InternalSensorRepoMapper,
	rMapper mapper.InternalRegionRepoMapper,
	tMapper mapper.InternalTreeRepoMapper,
) TreeClusterMappers {
	return TreeClusterMappers{
		mapper:       tcMapper,
		sensorMapper: sMapper,
		regionMapper: rMapper,
		treeMapper:   tMapper,
	}
}

var _ = (*TreeClusterRepository)(nil)

func NewTreeClusterRepository(s *store.Store, mappers TreeClusterMappers) *TreeClusterRepository {
	return &TreeClusterRepository{
		store:              s,
		TreeClusterMappers: mappers,
	}
}

func (r *TreeClusterRepository) Archive(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	_, err := r.store.ArchiveTreeCluster(ctx, id)
	if err != nil {
		log.Error("failed to archive tree cluster entity in db", "error", err, "cluster_id", id)
		return err
	}

	log.Debug("tree cluster entity archived successfully", "cluster_id", id)
	return nil
}

func (r *TreeClusterRepository) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	_, err := r.store.DeleteTreeCluster(ctx, id)
	if err != nil {
		log.Error("failed to delete tree cluster entity in db", "error", err, "cluster_id", id)
		return err
	}

	log.Debug("tree cluster entity deleted successfully", "cluster_id", id)
	return nil
}
