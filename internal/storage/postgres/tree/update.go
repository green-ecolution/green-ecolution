package tree

import (
	"context"

	"github.com/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/backend/internal/storage/postgres/store"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/pkg/errors"
)

func (r *TreeRepository) Update(ctx context.Context, id int32, updateFn func(*entities.Tree, storage.TreeRepository) (bool, error)) (*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	var updatedTree *entities.Tree

	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewTreeRepository(s, r.TreeMappers)

		tree, err := newRepo.GetByID(ctx, id)
		if err != nil {
			log.Error("failed to get tree entity from db", "error", err, "tree_id", id)
			return err
		}

		if updateFn == nil {
			return errors.New("updateFn is nil")
		}

		updated, err := updateFn(tree, newRepo)
		if err != nil {
			return err
		}

		if !updated {
			updatedTree = tree
			return nil
		}

		if err := newRepo.updateEntity(ctx, tree); err != nil {
			log.Error("failed to update tree entity in db", "error", err, "tree_id", id)
			return err
		}

		updatedTree, err = newRepo.GetByID(ctx, id)
		if err != nil {
			log.Error("failed to get updated tree entity from db", "error", err, "tree_id", id)
			return err
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	log.Debug("tree entity updated successfully in db", "tree_id", id)
	return updatedTree, nil
}

func (r *TreeRepository) updateEntity(ctx context.Context, t *entities.Tree) error {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(t.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", t.AdditionalInfo)
		return err
	}

	var treeClusterID *int32
	if t.TreeCluster != nil {
		treeClusterID = &t.TreeCluster.ID
	}

	var sensorID *string
	if t.Sensor != nil {
		sensorID = &t.Sensor.ID

		if err := r.store.UnlinkSensorIDFromTrees(ctx, sensorID); err != nil {
			return err
		}
	}

	args := sqlc.UpdateTreeParams{
		ID:                     t.ID,
		Species:                t.Species,
		PlantingYear:           t.PlantingYear,
		Number:                 t.Number,
		SensorID:               sensorID,
		TreeClusterID:          treeClusterID,
		WateringStatus:         sqlc.WateringStatus(t.WateringStatus),
		Description:            &t.Description,
		Provider:               &t.Provider,
		AdditionalInformations: additionalInfo,
		LastWatered:            utils.TimeToPgTimestamp(t.LastWatered),
	}

	if err := r.store.SetTreeLocation(ctx, &sqlc.SetTreeLocationParams{
		ID:        t.ID,
		Latitude:  t.Latitude,
		Longitude: t.Longitude,
	}); err != nil {
		return err
	}

	return r.store.UpdateTree(ctx, &args)
}
