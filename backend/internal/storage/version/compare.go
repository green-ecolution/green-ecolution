package version

import (
	"strings"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	goversion "github.com/hashicorp/go-version"
)

func CompareVersions(current, latest string) entities.VersionInfo {
	result := entities.VersionInfo{
		Current:         current,
		Latest:          latest,
		UpdateAvailable: false,
		IsDevelopment:   false,
		IsStage:         false,
		ReleaseURL:      "",
	}

	// Check for development or stage builds
	if isDevelopmentVersion(current) {
		result.IsDevelopment = true
		return result
	}

	if isStageVersion(current) {
		result.IsStage = true
		return result
	}

	// Try to compare versions
	currentVer, err := goversion.NewVersion(current)
	if err != nil {
		return result
	}

	latestVer, err := goversion.NewVersion(latest)
	if err != nil {
		return result
	}

	result.UpdateAvailable = latestVer.GreaterThan(currentVer)
	return result
}

func isDevelopmentVersion(v string) bool {
	return strings.Contains(v, "-g") ||
		strings.Contains(v, "-dirty") ||
		v == "development" ||
		v == "develop"
}

func isStageVersion(v string) bool {
	return strings.Contains(v, "-stage")
}
