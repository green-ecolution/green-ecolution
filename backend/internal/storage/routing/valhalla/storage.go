package valhalla

import (
	"log/slog"

	"github.com/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/backend/internal/storage"
)

func NewRepository(cfg *config.Config) (*storage.Repository, error) {
	repoCfg := &RouteRepoConfig{
		routing: cfg.Routing,
	}

	routingRepo, err := NewRouteRepo(repoCfg)
	if err != nil {
		slog.Error("failed to setup routing repository", "error", err, "service", "valhalla")
		return nil, err
	}

	slog.Info("successfully initialized routing repository", "service", "valhalla")
	return &storage.Repository{
		Routing: routingRepo,
	}, nil
}
