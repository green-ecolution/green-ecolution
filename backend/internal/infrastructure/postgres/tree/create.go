package tree

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"

	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func defaultTree() shared.Tree {
	return shared.Tree{
		TreeCluster:    nil,
		Species:        "",
		Number:         "",
		Sensor:         nil,
		PlantingYear:   shared.PlantingYear{},
		Coordinate:     shared.MustNewCoordinate(0, 0),
		WateringStatus: shared.WateringStatusUnknown,
		Description:    "",
		Provider:       "",
		AdditionalInfo: nil,
		LastWatered:    nil,
	}
}

func (r *TreeRepository) Create(ctx context.Context, createFn func(*shared.Tree, shared.TreeRepository) (bool, error)) (*shared.Tree, error) {
	log := logger.GetLogger(ctx)
	if createFn == nil {
		return nil, errors.New("createFn is nil")
	}

	var createdTree *shared.Tree
	err := r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewTreeRepository(s, r.TreeMappers)
		entity := defaultTree()

		created, err := createFn(&entity, newRepo)
		if err != nil {
			return err
		}

		if !created {
			return nil
		}

		if err := newRepo.validateTreeEntity(&entity); err != nil {
			return err
		}

		id, err := newRepo.createEntity(ctx, &entity)
		if err != nil {
			log.Error("failed to create tree entity in db", "error", err)
			return err
		}
		entity.ID = id

		createdTree, err = newRepo.GetByID(ctx, id)
		if err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	log.Debug("tree entity created successfully in db", "tree_id", createdTree.ID)
	return createdTree, nil
}

func (r *TreeRepository) createEntity(ctx context.Context, entity *shared.Tree) (int32, error) {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(entity.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", entity.AdditionalInfo)
		return -1, err
	}

	var treeClusterID *int32
	if entity.TreeCluster != nil {
		treeClusterID = &entity.TreeCluster.ID
	}

	var sensorID *string
	if entity.Sensor != nil {
		s := entity.Sensor.ID.String()
		sensorID = &s
		if err := r.store.UnlinkSensorIDFromTrees(ctx, sensorID); err != nil {
			return -1, err
		}
	}

	args := sqlc.CreateTreeParams{
		TreeClusterID:          treeClusterID,
		Species:                entity.Species,
		SensorID:               sensorID,
		PlantingYear:           entity.PlantingYear.Year(),
		Latitude:               entity.Coordinate.Latitude(),
		Longitude:              entity.Coordinate.Longitude(),
		WateringStatus:         sqlc.WateringStatus(entity.WateringStatus),
		Description:            &entity.Description,
		Number:                 entity.Number,
		Provider:               &entity.Provider,
		AdditionalInformations: additionalInfo,
	}

	id, err := r.store.CreateTree(ctx, &args)
	if err != nil {
		return -1, err
	}

	if err := r.store.SetTreeLocation(ctx, &sqlc.SetTreeLocationParams{
		ID:        id,
		Latitude:  entity.Coordinate.Latitude(),
		Longitude: entity.Coordinate.Longitude(),
	}); err != nil {
		return -1, err
	}

	return id, nil
}

func (r *TreeRepository) validateTreeEntity(tree *shared.Tree) error {
	if tree == nil {
		return errors.New("tree is nil")
	}
	return nil
}
