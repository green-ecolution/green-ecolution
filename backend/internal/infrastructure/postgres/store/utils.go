package store

import (
	"context"
	"errors"

	"github.com/jackc/pgx/v5"

	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

var (
	regionMapper = mapper.InternalRegionRepoMapperImpl{}
	treeMapper   = mapper.InternalTreeRepoMapperImpl{}
	sensorMapper = mapper.InternalSensorRepoMapperImpl{}
)

// This function is required as soon as you want to add data to the tree cluster object
// from the database, e.g. the linked region or the linked trees.
// As this function is required in different repositories, it has been outsourced.
func (s *Store) MapClusterFields(ctx context.Context, tc *entities.TreeCluster) error {
	if err := s.mapRegion(ctx, tc); err != nil {
		return err
	}

	if err := s.mapTrees(ctx, tc); err != nil {
		return err
	}

	return nil
}

// This function is required as soon as you want to add the data to the sensor object
func (s *Store) MapSensorFields(ctx context.Context, sn *entities.Sensor) error {
	var err error
	sn.LatestData, err = s.GetLatestSensorDataBySensorID(ctx, sn.ID.String())

	var entityNotFoundErr entities.ErrEntityNotFound
	if err != nil && !errors.As(err, &entityNotFoundErr) {
		return err
	}

	return nil
}

// This function provides the latest data from a specific sensor
func (s *Store) GetLatestSensorDataBySensorID(ctx context.Context, id string) (*entities.SensorData, error) {
	row, err := s.GetLatestSensorDataByID(ctx, id)
	if err != nil {
		return nil, s.MapError(err, sqlc.SensorDatum{})
	}

	domainData, err := sensorMapper.FromSqlSensorData(row)
	if err != nil {
		return nil, errors.Join(err, errors.New("failed to map sensor data"))
	}

	return domainData, nil
}

func (s *Store) mapRegion(ctx context.Context, tc *entities.TreeCluster) error {
	region, err := s.getRegionByTreeClusterID(ctx, tc.ID)
	if err != nil {
		// If region is not found, we can still return the tree cluster
		if !errors.Is(err, entities.ErrRegionNotFound) {
			return err
		}
	}
	tc.Region = region

	return nil
}

func (s *Store) mapTrees(ctx context.Context, tc *entities.TreeCluster) error {
	trees, err := s.getLinkedTreesByTreeClusterID(ctx, tc.ID)
	if err != nil {
		return err
	}
	tc.Trees = trees

	return nil
}

func (s *Store) getRegionByTreeClusterID(ctx context.Context, id int32) (*entities.Region, error) {
	row, err := s.GetRegionByTreeClusterID(ctx, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, entities.ErrRegionNotFound
		}
		return nil, err
	}

	return regionMapper.FromSql(row), nil
}

func (s *Store) getLinkedTreesByTreeClusterID(ctx context.Context, id int32) ([]*entities.Tree, error) {
	log := logger.GetLogger(ctx)
	rows, err := s.GetLinkedTreesByTreeClusterID(ctx, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return []*entities.Tree{}, nil
		}
		return nil, err
	}

	trees, err := treeMapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return trees, nil
}
