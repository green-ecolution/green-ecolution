package mapper

import (
	"time"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToTime github.com/green-ecolution/green-ecolution/backend/internal/utils:URLToURL github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeDurationToTimeDuration github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToTime github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToURL github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToNetIP
// goverter:extend github.com/green-ecolution/green-ecolution/backend/internal/utils:StringToDuration github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeToString github.com/green-ecolution/green-ecolution/backend/internal/utils:NetURLToString github.com/green-ecolution/green-ecolution/backend/internal/utils:NetIPToString github.com/green-ecolution/green-ecolution/backend/internal/utils:TimeDurationToString
// goverter:extend MapCenter MapBbox MapServiceStatusItems MapServiceStatusItemsReverse
type InfoHTTPMapper interface {
	ToResponse(src *domain.App) *entities.AppInfoResponse
	ServerToResponse(src *domain.Server) *entities.ServerResponse
	ServicesToResponse(src *domain.Services) *entities.ServicesResponse
}

func MapCenter(src []float64) []float64 {
	return src
}

func MapBbox(src []float64) []float64 {
	return src
}

func MapServiceStatusItems(src []domain.ServiceStatus) []entities.ServiceStatusResponse {
	result := make([]entities.ServiceStatusResponse, len(src))
	for i, s := range src {
		result[i] = entities.ServiceStatusResponse{
			Name:           s.Name,
			Enabled:        s.Enabled,
			Healthy:        s.Healthy,
			ResponseTimeMs: float64(s.ResponseTime.Nanoseconds()) / 1e6,
			LastChecked:    formatTimeISO8601(s.LastChecked),
			Message:        s.Message,
		}
	}
	return result
}

func formatTimeISO8601(t time.Time) string {
	if t.IsZero() {
		return ""
	}
	return t.Format(time.RFC3339)
}

func MapServiceStatusItemsReverse(src []entities.ServiceStatusResponse) []domain.ServiceStatus {
	result := make([]domain.ServiceStatus, len(src))
	for i, s := range src {
		var lastChecked time.Time
		if s.LastChecked != "" {
			lastChecked, _ = time.Parse(time.RFC3339, s.LastChecked)
		}
		result[i] = domain.ServiceStatus{
			Name:         s.Name,
			Enabled:      s.Enabled,
			Healthy:      s.Healthy,
			ResponseTime: time.Duration(s.ResponseTimeMs * 1e6),
			LastChecked:  lastChecked,
			Message:      s.Message,
		}
	}
	return result
}
