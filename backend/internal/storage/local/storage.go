package local

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/local/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/version"
)

func NewRepository(cfg *config.Config) (*storage.Repository, *info.InfoRepository, error) {
	versionRepo := version.NewGitHubVersionRepository()
	infoRepo, err := info.NewInfoRepository(cfg, versionRepo, nil)
	if err != nil {
		slog.Debug("failed to setup info repository", "error", err)
		return nil, nil, err
	}

	slog.Info("successfully initialized info repository")
	return &storage.Repository{
		Info: infoRepo,
	}, infoRepo, nil
}
