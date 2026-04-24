package mapper

import (
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func InfoToResponse(source *info.App) *entities.AppInfoResponse {
	if source == nil {
		return nil
	}
	return &entities.AppInfoResponse{
		Version:     source.Version,
		VersionInfo: versionInfoToResponse(source.VersionInfo),
		BuildTime:   utils.TimeToString(source.BuildTime),
		GoVersion:   source.GoVersion,
		Git:         gitToResponse(source.Git),
		Map:         mapToResponse(source.Map),
	}
}

func InfoServerToResponse(source *info.Server) *entities.ServerResponse {
	if source == nil {
		return nil
	}
	return &entities.ServerResponse{
		OS:        source.OS,
		Arch:      source.Arch,
		Hostname:  source.Hostname,
		URL:       utils.NetURLToString(source.URL),
		IP:        utils.NetIPToString(source.IP),
		Port:      source.Port,
		Interface: source.Interface,
		Uptime:    utils.TimeDurationToString(source.Uptime),
	}
}

func InfoServicesToResponse(source *info.Services) *entities.ServicesResponse {
	if source == nil {
		return nil
	}
	return &entities.ServicesResponse{
		Items: MapServiceStatusItems(source.Items),
	}
}

func versionInfoToResponse(source info.VersionInfo) entities.VersionInfoResponse {
	return entities.VersionInfoResponse{
		Current:         source.Current,
		Latest:          source.Latest,
		UpdateAvailable: source.UpdateAvailable,
		IsDevelopment:   source.IsDevelopment,
		IsStage:         source.IsStage,
		ReleaseURL:      source.ReleaseURL,
	}
}

func gitToResponse(source info.Git) entities.GitResponse {
	return entities.GitResponse{
		Branch:     source.Branch,
		Commit:     source.Commit,
		Repository: utils.NetURLToString(source.Repository),
	}
}

func mapToResponse(source info.Map) entities.MapResponse {
	return entities.MapResponse{
		Center: MapBbox(source.Center),
		BBox:   MapBbox(source.BBox),
	}
}

func MapCenter(src []float64) []float64 {
	return src
}

func MapBbox(src []float64) []float64 {
	return src
}

func MapServiceStatusItems(src []info.ServiceStatus) []entities.ServiceStatusResponse {
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

func MapServiceStatusItemsReverse(src []entities.ServiceStatusResponse) []info.ServiceStatus {
	result := make([]info.ServiceStatus, len(src))
	for i, s := range src {
		var lastChecked time.Time
		if s.LastChecked != "" {
			lastChecked, _ = time.Parse(time.RFC3339, s.LastChecked)
		}
		result[i] = info.ServiceStatus{
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
