package store

import (
	"context"
	"errors"

	"github.com/jackc/pgx/v5"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/cluster"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
)

var (
	regionMapper = mapper.InternalRegionRepoMapperImpl{}
	sensorMapper = mapper.InternalSensorRepoMapperImpl{}
)

// This function is required as soon as you want to add data to the tree cluster object
// from the database, e.g. the linked region or the linked trees.
// As this function is required in different repositories, it has been outsourced.
func (s *Store) MapClusterFields(ctx context.Context, tc *cluster.TreeCluster) error {
	if err := s.mapRegion(ctx, tc); err != nil {
		return err
	}

	if err := s.mapTrees(ctx, tc); err != nil {
		return err
	}

	return nil
}

// This function is required as soon as you want to add the data to the sensor object
func (s *Store) MapSensorFields(ctx context.Context, sn *sensor.Sensor) error {
	var err error
	sn.LatestData, err = s.GetLatestSensorDataBySensorID(ctx, sn.ID.String())

	var entityNotFoundErr shared.ErrEntityNotFound
	if err != nil && !errors.As(err, &entityNotFoundErr) {
		return err
	}

	return nil
}

// This function provides the latest data from a specific sensor
func (s *Store) GetLatestSensorDataBySensorID(ctx context.Context, id string) (*sensor.SensorData, error) {
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

func (s *Store) mapRegion(ctx context.Context, tc *cluster.TreeCluster) error {
	r, err := s.getRegionByTreeClusterID(ctx, tc.ID)
	if err != nil {
		// If region is not found, we can still return the tree cluster
		if !errors.Is(err, region.ErrNotFound) {
			return err
		}
	}
	if r != nil {
		tc.RegionID = &r.ID
	}

	return nil
}

func (s *Store) mapTrees(ctx context.Context, tc *cluster.TreeCluster) error {
	treeIDs, err := s.getLinkedTreeIDsByTreeClusterID(ctx, tc.ID)
	if err != nil {
		return err
	}
	tc.TreeIDs = treeIDs

	return nil
}

func (s *Store) getRegionByTreeClusterID(ctx context.Context, id int32) (*region.Region, error) {
	row, err := s.GetRegionByTreeClusterID(ctx, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, region.ErrNotFound
		}
		return nil, err
	}

	return regionMapper.FromSql(row), nil
}

func (s *Store) getLinkedTreeIDsByTreeClusterID(ctx context.Context, id int32) ([]int32, error) {
	rows, err := s.GetLinkedTreesByTreeClusterID(ctx, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return []int32{}, nil
		}
		return nil, err
	}

	ids := make([]int32, 0, len(rows))
	for _, row := range rows {
		ids = append(ids, row.ID)
	}

	return ids, nil
}
