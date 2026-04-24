package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/sensor"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/tree"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalTreeRepoMapper interface {
	FromSql(*sqlc.Tree) (*tree.Tree, error)
	FromSqlList([]*sqlc.Tree) ([]*tree.Tree, error)
}

type InternalTreeRepoMapperImpl struct{}

func (c *InternalTreeRepoMapperImpl) FromSql(source *sqlc.Tree) (*tree.Tree, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	sensorID, err := mapSensorID(source.SensorID)
	if err != nil {
		return nil, err
	}

	return &tree.Tree{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		TreeClusterID:  source.TreeClusterID,
		SensorID:       sensorID,
		PlantingYear:   mapPlantingYear(source.PlantingYear),
		Species:        source.Species,
		Number:         source.Number,
		Coordinate:     shared.MustNewCoordinate(source.Latitude, source.Longitude),
		WateringStatus: MapWateringStatus(source.WateringStatus),
		Description:    utils.StringPtrToString(source.Description),
		LastWatered:    timePtrToTimePtr(source.LastWatered),
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}, nil
}

func (c *InternalTreeRepoMapperImpl) FromSqlList(source []*sqlc.Tree) ([]*tree.Tree, error) {
	return utils.MapSliceErr(source, c.FromSql)
}

func mapSensorID(id *string) (*sensor.SensorID, error) {
	if id == nil {
		return nil, nil
	}
	sid, err := sensor.NewSensorID(*id)
	if err != nil {
		return nil, err
	}
	return &sid, nil
}

func mapPlantingYear(year int32) tree.PlantingYear {
	if year <= 0 {
		return tree.PlantingYear{}
	}
	return tree.MustNewPlantingYear(year)
}
