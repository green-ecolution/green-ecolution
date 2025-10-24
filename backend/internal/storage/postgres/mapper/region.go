package mapper

import (
	"github.com/green-ecolution/backend/internal/entities"
	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
type InternalRegionRepoMapper interface {
	FromSql(src *sqlc.Region) *entities.Region
	FromSqlList(src []*sqlc.Region) []*entities.Region
}
