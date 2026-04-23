package openrouteservice

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

func NewRepository(cfg *config.Config) (*shared.Repository, error) {
	repoCfg := &RouteRepoConfig{
		routing: cfg.Routing,
	}

	routingRepo, err := NewRouteRepo(repoCfg)
	if err != nil {
		slog.Error("error creating routing repo", "error", err)
		return nil, err
	}
	return &shared.Repository{
		Routing: routingRepo,
	}, nil
}
