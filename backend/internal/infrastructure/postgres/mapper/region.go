package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type InternalRegionRepoMapper interface {
	FromSql(src *sqlc.Region) *shared.Region
	FromSqlList(src []*sqlc.Region) []*shared.Region
}

type InternalRegionRepoMapperImpl struct{}

func (c *InternalRegionRepoMapperImpl) FromSql(source *sqlc.Region) *shared.Region {
	if source == nil {
		return nil
	}
	return &shared.Region{
		ID:        source.ID,
		CreatedAt: source.CreatedAt,
		UpdatedAt: source.UpdatedAt,
		Name:      source.Name,
	}
}

func (c *InternalRegionRepoMapperImpl) FromSqlList(source []*sqlc.Region) []*shared.Region {
	return utils.MapSlice(source, c.FromSql)
}
