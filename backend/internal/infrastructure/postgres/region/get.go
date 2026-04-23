package region

import (
	"context"
	"errors"
	"fmt"

	"github.com/jackc/pgx/v5"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

func (r *RegionRepository) GetAll(ctx context.Context) ([]*entities.Region, int64, error) {
	log := logger.GetLogger(ctx)
	page, limit, err := pagination.GetValues(ctx)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Region{})
	}

	totalCount, err := r.store.GetAllRegionsCount(ctx)
	if err != nil {
		log.Debug("failed to get total regions count in db", "error", err)
		return nil, 0, r.store.MapError(err, sqlc.Region{})
	}

	if totalCount == 0 {
		return []*entities.Region{}, 0, nil
	}

	if limit == -1 {
		limit = int32(totalCount)
		page = 1
	}

	rows, err := r.store.GetAllRegions(ctx, &sqlc.GetAllRegionsParams{
		Limit:  limit,
		Offset: (page - 1) * limit,
	})

	if err != nil {
		log.Debug("failed to get regions in db", "error", err)
		return nil, 0, r.store.MapError(err, sqlc.Region{})
	}

	return r.mapper.FromSqlList(rows), totalCount, nil
}

func (r *RegionRepository) GetByID(ctx context.Context, id int32) (*entities.Region, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetRegionById(ctx, id)
	if err != nil {
		log.Debug("failed to get region by id", "error", err, "region_id", id)
		return nil, r.store.MapError(err, sqlc.Region{})
	}

	return r.mapper.FromSql(row), nil
}

func (r *RegionRepository) GetByPoint(ctx context.Context, coord entities.Coordinate) (*entities.Region, error) {
	log := logger.GetLogger(ctx)
	p := fmt.Sprintf("POINT(%f %f)", coord.Longitude(), coord.Latitude())
	region, err := r.store.GetRegionByPoint(ctx, p)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			log.Debug("no existing region in given coordinates", "error", err, "latitude", coord.Latitude(), "longitude", coord.Longitude())
			return nil, nil
		}
		log.Debug("failed to translate point to region", "error", err, "latitude", coord.Latitude(), "longitude", coord.Longitude())
		return nil, r.store.MapError(err, sqlc.Region{})
	}

	return r.mapper.FromSql(region), nil
}
