package tree

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"

	"github.com/pkg/errors"

	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func (r *TreeRepository) Update(ctx context.Context, id int32, entity *tree.Tree) (*tree.Tree, error) {
	log := logger.GetLogger(ctx)
	if entity == nil {
		return nil, errors.New("entity is nil")
	}

	var updatedTree *tree.Tree

	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewTreeRepository(s, r.TreeMappers)

		if _, err := newRepo.GetByID(ctx, id); err != nil {
			log.Error("failed to get tree entity from db", "error", err, "tree_id", id)
			return err
		}

		entity.ID = id
		if err := newRepo.updateEntity(ctx, entity); err != nil {
			log.Error("failed to update tree entity in db", "error", err, "tree_id", id)
			return err
		}

		var err error
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

func (r *TreeRepository) updateEntity(ctx context.Context, t *tree.Tree) error {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(t.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", t.AdditionalInfo)
		return err
	}

	treeClusterID := t.TreeClusterID

	var sensorID *string
	if t.SensorID != nil {
		s := t.SensorID.String()
		sensorID = &s

		if err := r.store.UnlinkSensorIDFromTrees(ctx, sensorID); err != nil {
			return err
		}
	}

	args := sqlc.UpdateTreeParams{
		ID:                     t.ID,
		Species:                t.Species,
		PlantingYear:           t.PlantingYear.Year(),
		Number:                 t.Number,
		SensorID:               sensorID,
		TreeClusterID:          treeClusterID,
		WateringStatus:         sqlc.WateringStatus(t.WateringStatus),
		Description:            &t.Description,
		Provider:               &t.Provider,
		AdditionalInformations: additionalInfo,
		LastWatered:            t.LastWatered,
	}

	if err := r.store.SetTreeLocation(ctx, &sqlc.SetTreeLocationParams{
		ID:        t.ID,
		Latitude:  t.Coordinate.Latitude(),
		Longitude: t.Coordinate.Longitude(),
	}); err != nil {
		return err
	}

	return r.store.UpdateTree(ctx, &args)
}
