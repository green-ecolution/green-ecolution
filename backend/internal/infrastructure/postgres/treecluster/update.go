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

func (r *TreeClusterRepository) Update(ctx context.Context, id int32, entity *cluster.TreeCluster) error {
	log := logger.GetLogger(ctx)
	if entity == nil {
		return errors.New("entity is nil")
	}

	return r.store.WithTx(ctx, func(s *store.Store) error {
		newRepo := NewTreeClusterRepository(s, r.TreeClusterMappers)

		if _, err := newRepo.GetByID(ctx, id); err != nil {
			return err
		}

		entity.ID = id
		if err := newRepo.updateEntity(ctx, entity); err != nil {
			log.Error("failed to update tree cluster entity in db", "error", err, "cluster_id", id)
			return err
		}

		log.Debug("tree cluster updated successfully in db", "cluster_id", id)
		return nil
	})
}

func (r *TreeClusterRepository) updateEntity(ctx context.Context, tc *cluster.TreeCluster) error {
	log := logger.GetLogger(ctx)
	additionalInfo, err := utils.MapAdditionalInfoToByte(tc.AdditionalInfo)
	if err != nil {
		log.Debug("failed to marshal additional informations to byte array", "error", err, "additional_info", tc.AdditionalInfo)
		return err
	}

	args := sqlc.UpdateTreeClusterParams{
		ID:                     tc.ID,
		RegionID:               tc.RegionID,
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

	if len(tc.TreeIDs) > 0 {
		if err := r.LinkTreesToCluster(ctx, tc.ID, tc.TreeIDs); err != nil {
			return err
		}
	}

	if tc.Coordinate == nil {
		if err := r.store.RemoveTreeClusterLocation(ctx, tc.ID); err != nil {
			return err
		}
	} else {
		lat := tc.Coordinate.Latitude()
		lng := tc.Coordinate.Longitude()
		locationArgs := sqlc.SetTreeClusterLocationParams{
			ID:        tc.ID,
			Latitude:  &lat,
			Longitude: &lng,
		}
		if err := r.store.SetTreeClusterLocation(ctx, &locationArgs); err != nil {
			return err
		}
	}

	return r.store.UpdateTreeCluster(ctx, &args)
}
