package local

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/local/info"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/version"
)

func NewRepository(cfg *config.Config) (*entities.Repository, *info.InfoRepository, error) {
	versionRepo := version.NewGitHubVersionRepository()
	infoRepo, err := info.NewInfoRepository(cfg, versionRepo, nil)
	if err != nil {
		slog.Debug("failed to setup info repository", "error", err)
		return nil, nil, err
	}

	slog.Info("successfully initialized info repository")
	return &entities.Repository{
		Info: infoRepo,
	}, infoRepo, nil
}
