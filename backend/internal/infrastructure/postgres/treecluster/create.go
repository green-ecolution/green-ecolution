package treecluster

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func defaultTreeCluster() *cluster.TreeCluster {
	return &cluster.TreeCluster{
		RegionID:       nil,
		Address:        "",
		Description:    "",
		MoistureLevel:  0,
		Coordinate:     nil,
		WateringStatus: shared.WateringStatusUnknown,
		SoilCondition:  cluster.TreeSoilConditionUnknown,
		Archived:       false,
		LastWatered:    nil,
		TreeIDs:        make([]int32, 0),
		Name:           "",
		Provider:       "",
		AdditionalInfo: nil,
	}
}

func (r *TreeClusterRepository) Create(ctx context.Context, createFn func(*cluster.TreeCluster, cluster.TreeClusterRepository) (bool, error)) (*cluster.TreeCluster, error) {
	log := logger.GetLogger(ctx)
	if createFn == nil {
		return nil, errors.New("createFn is nil")
	}

	var createdTc *cluster.TreeCluster
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewTreeClusterRepository(s, r.TreeClusterMappers)
		entity := defaultTreeCluster()
		created, err := createFn(entity, newRepo)
		if err != nil {
			return err
		}

		if !created {
			return nil
		}

		if err := newRepo.validateTreeClusterEntity(entity); err != nil {
			return err
		}

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
		log.Error("failed to update tree cluster entity in db", "error", err)
		return nil, err
	}

	if createdTc != nil {
		log.Debug("tree cluster entity created successfully", "cluster_id", createdTc.ID)
	} else {
		log.Debug("tree cluster should not be created. cancel transaction")
	}

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
