package local

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/local/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/version"
)

func NewRepository(cfg *config.Config) (*storage.Repository, error) {
	versionRepo := version.NewGitHubVersionRepository()
	infoRepo, err := info.NewInfoRepository(cfg, versionRepo)
	if err != nil {
		slog.Debug("failed to setup info repository", "error", err)
		return nil, err
	}

	slog.Info("successfully initialized info repository")
	return &storage.Repository{
		Info: infoRepo,
	}, nil
}
