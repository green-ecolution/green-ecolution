package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalRegionRepoMapper interface {
	FromSql(src *sqlc.Region) *entities.Region
	FromSqlList(src []*sqlc.Region) []*entities.Region
}

type InternalRegionRepoMapperImpl struct{}

func (c *InternalRegionRepoMapperImpl) FromSql(source *sqlc.Region) *entities.Region {
	if source == nil {
		return nil
	}
	return &entities.Region{
		ID:        source.ID,
		CreatedAt: utils.TimeToTime(source.CreatedAt),
		UpdatedAt: utils.TimeToTime(source.UpdatedAt),
		Name:      source.Name,
	}
}

func (c *InternalRegionRepoMapperImpl) FromSqlList(source []*sqlc.Region) []*entities.Region {
	return utils.MapSlice(source, c.FromSql)
}
