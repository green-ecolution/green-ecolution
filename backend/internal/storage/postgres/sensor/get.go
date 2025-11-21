package sensor

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/pagination"
)

func (r *SensorRepository) GetAll(ctx context.Context, query entities.Query) ([]*entities.Sensor, int64, error) {
	log := logger.GetLogger(ctx)
	page, limit, err := pagination.GetValues(ctx)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Sensor{})
	}

	totalCount, err := r.GetCount(ctx, query)
	if err != nil {
		return nil, 0, r.store.MapError(err, sqlc.Sensor{})
	}

	if totalCount == 0 {
		return []*entities.Sensor{}, 0, nil
	}

	if limit == -1 {
		limit = int32(totalCount)
		page = 1
	}

	rows, err := r.store.GetAllSensors(ctx, &sqlc.GetAllSensorsParams{
		Provider: query.Provider,
		Limit:    limit,
		Offset:   (page - 1) * limit,
	})
	if err != nil {
		log.Debug("failed to get sensors in db", "error", err)
		return nil, 0, r.store.MapError(err, sqlc.Sensor{})
	}

	data, err := r.mapper.FromSqlList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, 0, err
	}

	for _, sn := range data {
		if err := r.store.MapSensorFields(ctx, sn); err != nil {
			return nil, 0, err
		}
	}

	return data, totalCount, nil
}

func (r *SensorRepository) GetCount(ctx context.Context, query entities.Query) (int64, error) {
	log := logger.GetLogger(ctx)
	totalCount, err := r.store.GetAllSensorsCount(ctx, query.Provider)
	if err != nil {
		log.Debug("failed to get total sensor count in db", "error", err)
		return 0, err
	}

	return totalCount, nil
}

func (r *SensorRepository) GetAllDataByID(ctx context.Context, id string) ([]*entities.SensorData, error) {
	log := logger.GetLogger(ctx)

	_, err := r.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get sensor in db", "error", err, "sensor_id", id)
		return nil, r.store.MapError(err, sqlc.Sensor{})
	}

	rows, err := r.store.GetAllSensorDataByID(ctx, id)
	if err != nil {
		log.Debug("failed to get all sensor data by sensor id in db", "error", err, "sensor_id", id)
		return nil, r.store.MapError(err, sqlc.Sensor{})
	}

	data, err := r.mapper.FromSqlSensorDataList(rows)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	return data, nil
}

func (r *SensorRepository) GetByID(ctx context.Context, id string) (*entities.Sensor, error) {
	log := logger.GetLogger(ctx)
	row, err := r.store.GetSensorByID(ctx, id)
	if err != nil {
		log.Debug("failed to get sensor by id in db", "error", err, "sensor_id", id)
		return nil, r.store.MapError(err, sqlc.Sensor{})
	}

	data, err := r.mapper.FromSql(row)
	if err != nil {
		log.Debug("failed to convert entity", "error", err)
		return nil, err
	}

	if err := r.store.MapSensorFields(ctx, data); err != nil {
		return nil, err
	}

	return data, nil
}

func (r *SensorRepository) GetLatestSensorDataBySensorID(ctx context.Context, id string) (*entities.SensorData, error) {
	log := logger.GetLogger(ctx)
	data, err := r.store.GetLatestSensorDataBySensorID(ctx, id)
	if err != nil {
		log.Debug("failed to get latest sensor data by sensor id in db", "error", err, "sensor_id", id)
		return nil, r.store.MapError(err, sqlc.Sensor{})
	}

	return data, nil
}
