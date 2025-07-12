package treecluster

import (
	"context"
	"errors"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/green-ecolution/backend/internal/utils"
)

func (r *TreeClusterRepository) Update(ctx context.Context, id int32, updateFn func(*entities.TreeCluster, storage.TreeClusterRepository) (bool, error)) error {
	log := logger.GetLogger(ctx)
	return r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewTreeClusterRepository(s, r.TreeClusterMappers)
		tc, err := newRepo.GetByID(ctx, id)
		if err != nil {
			return err
		}

		if updateFn == nil {
			return errors.New("updateFn is nil")
		}

		updated, err := updateFn(tc, newRepo)
		if err != nil {
			return err
		}

		if !updated {
			return nil
		}

		if err := newRepo.updateEntity(ctx, tc); err != nil {
			log.Error("failed to update tree cluster entity in db", "error", err, "cluster_id", id)
		}

		log.Debug("tree cluster updated successfully in db", "cluster_id", id)
		return nil
	})
}

func (r *TreeClusterRepository) updateEntity(ctx context.Context, tc *entities.TreeCluster) error {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(tc.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", tc.AdditionalInfo)
		return err
	}

	var regionID *int32
	if tc.Region != nil {
		regionID = &tc.Region.ID
	}
	args := sqlc.UpdateTreeClusterParams{
		ID:                     tc.ID,
		RegionID:               regionID,
		Address:                tc.Address,
		Description:            tc.Description,
		MoistureLevel:          tc.MoistureLevel,
		WateringStatus:         sqlc.WateringStatus(tc.WateringStatus),
		SoilCondition:          sqlc.TreeSoilCondition(tc.SoilCondition),
		LastWatered:            tc.LastWatered,
		Archived:               tc.Archived,
		Name:                   tc.Name,
		Provider:               &tc.Provider,
		AdditionalInformations: additionalInfo,
	}

	if _, err := r.store.UnlinkTreeClusterID(ctx, &tc.ID); err != nil {
		log.Error("failed to unlink tree cluster from trees", "error", err, "cluster_id", tc.ID)
		return err
	}

	if len(tc.Trees) > 0 {
		treeIDs := utils.Map(tc.Trees, func(t *entities.Tree) int32 {
			return t.ID
		})

		if err := r.LinkTreesToCluster(ctx, tc.ID, treeIDs); err != nil {
			return err
		}
	}

	if tc.Latitude == nil || tc.Longitude == nil {
		if err := r.store.RemoveTreeClusterLocation(ctx, tc.ID); err != nil {
			return err
		}
	} else {
		locationArgs := sqlc.SetTreeClusterLocationParams{
			ID:        tc.ID,
			Latitude:  tc.Latitude,
			Longitude: tc.Longitude,
		}
		if err := r.store.SetTreeClusterLocation(ctx, &locationArgs); err != nil {
			return err
		}
	}

	return r.store.UpdateTreeCluster(ctx, &args)
}
