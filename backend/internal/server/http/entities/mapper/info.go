package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTime github.com/green-ecolution/green-ecolution/backend/internal/utils:URLToURL github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeDurationToTimeDuration github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToTime github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToURL github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToNetIP
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToDuration github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToString github.com/green-ecolution/green-ecolution/backend/internal/utils:NetURLToString github.com/green-ecolution/green-ecolution/backend/internal/utils:NetIPToString github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeDurationToString
// goverter:extend MapCenter MapBbox
type InfoHTTPMapper interface {
	ToResponse(src *domain.App) *entities.AppInfoResponse
	FromResponse(src *entities.AppInfoResponse) *domain.App
}

func MapCenter(src [2]float64) [2]float64 {
	return src
}

func MapBbox(src [4]float64) [4]float64 {
	return src
}
