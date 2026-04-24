package treecluster

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func (r *TreeClusterRepository) Create(ctx context.Context, entity *cluster.TreeCluster) (*cluster.TreeCluster, error) {
	log := logger.GetLogger(ctx)
	if entity == nil {
		return nil, errors.New("entity is nil")
	}

	if err := r.validateTreeClusterEntity(entity); err != nil {
		return nil, err
	}

	var createdTc *cluster.TreeCluster
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewTreeClusterRepository(s, r.TreeClusterMappers)

		id, err := newRepo.createEntity(ctx, entity)
		if err != nil {
			return err
		}
		createdTc, err = newRepo.GetByID(ctx, id)
		if err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		log.Error("failed to create tree cluster entity in db", "error", err)
		return nil, err
	}

	log.Debug("tree cluster entity created successfully", "cluster_id", createdTc.ID)
	return createdTc, nil
}

func (r *TreeClusterRepository) createEntity(ctx context.Context, entity *cluster.TreeCluster) (int32, error) {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(entity.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", entity.AdditionalInfo)
		return -1, err
	}

	args := sqlc.CreateTreeClusterParams{
		RegionID:               entity.RegionID,
		Address:                entity.Address,
		Description:            entity.Description,
		MoistureLevel:          entity.MoistureLevel,
		WateringStatus:         sqlc.WateringStatus(entity.WateringStatus),
		SoilCondition:          sqlc.TreeSoilCondition(entity.SoilCondition),
		Name:                   entity.Name,
		Provider:               &entity.Provider,
		AdditionalInformations: additionalInfo,
	}

	id, err := r.store.CreateTreeCluster(ctx, &args)
	if err != nil {
		return -1, err
	}

	if len(entity.TreeIDs) > 0 {
		err = r.LinkTreesToCluster(ctx, id, entity.TreeIDs)
		if err != nil {
			return -1, err
		}
	}

	if entity.Coordinate != nil {
		lat := entity.Coordinate.Latitude()
		lng := entity.Coordinate.Longitude()
		err = r.store.SetTreeClusterLocation(ctx, &sqlc.SetTreeClusterLocationParams{
			ID:        id,
			Latitude:  &lat,
			Longitude: &lng,
		})
		if err != nil {
			return -1, err
		}
	}

	return id, nil
}

func (r *TreeClusterRepository) validateTreeClusterEntity(tc *cluster.TreeCluster) error {
	if tc == nil {
		return errors.New("tree cluster is nil")
	}

	if tc.Name == "" {
		return errors.New("tree cluster name is empty")
	}

	return nil
}

func (r *TreeClusterRepository) LinkTreesToCluster(ctx context.Context, treeClusterID int32, treeIDs []int32) error {
	args := &sqlc.LinkTreesToTreeClusterParams{
		Column1:       treeIDs,
		TreeClusterID: &treeClusterID,
	}
	return r.store.LinkTreesToTreeCluster(ctx, args)
}
