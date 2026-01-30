package version

import (
	"context"
	"time"
)

type LatestVersionInfo struct {
	Version     string
	ReleaseURL  string
	PublishedAt time.Time
}

type VersionRepository interface {
	GetLatestVersion(ctx context.Context) (*LatestVersionInfo, error)
}
