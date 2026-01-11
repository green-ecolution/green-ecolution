package streamlet

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
)

func NewRepository(cfg *config.Config) (*storage.Repository, error) {
	routingRepo, err := NewStreamletClient(
		WithHostURL(cfg.Routing.Valhalla.Host),
	)
	if err != nil {
		slog.Error("failed to setup routing repository", "error", err, "service", "valhalla")
		return nil, err
	}

	slog.Info("successfully initialized routing repository", "service", "valhalla")
	return &storage.Repository{
		Routing: routingRepo,
	}, nil
}
