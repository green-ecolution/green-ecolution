package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalTreeRepoMapper interface {
	FromSql(*sqlc.Tree) (*entities.Tree, error)
	FromSqlList([]*sqlc.Tree) ([]*entities.Tree, error)
}

type InternalTreeRepoMapperImpl struct{}

func (c *InternalTreeRepoMapperImpl) FromSql(source *sqlc.Tree) (*entities.Tree, error) {
	if source == nil {
		return nil, nil
	}
	additionalInfo, err := utils.MapAdditionalInfo(source.AdditionalInformations)
	if err != nil {
		return nil, err
	}
	return &entities.Tree{
		ID:             source.ID,
		CreatedAt:      source.CreatedAt,
		UpdatedAt:      source.UpdatedAt,
		PlantingYear:   source.PlantingYear,
		Species:        source.Species,
		Number:         source.Number,
		Latitude:       source.Latitude,
		Longitude:      source.Longitude,
		WateringStatus: MapWateringStatus(source.WateringStatus),
		Description:    utils.StringPtrToString(source.Description),
		LastWatered:    timePtrToTimePtr(source.LastWatered),
		Provider:       utils.StringPtrToString(source.Provider),
		AdditionalInfo: additionalInfo,
	}, nil
}

func (c *InternalTreeRepoMapperImpl) FromSqlList(source []*sqlc.Tree) ([]*entities.Tree, error) {
	return utils.MapSliceErr(source, c.FromSql)
}
