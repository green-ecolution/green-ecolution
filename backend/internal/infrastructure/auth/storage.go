package auth

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/auth/keycloak"
)

func NewRepository(cfg *config.IdentityAuthConfig) *shared.Repository {
	authRepo := keycloak.NewKeycloakRepository(cfg)
	slog.Info("successfully initialized auth repository", "service", "keycloak")

	var userRepo shared.UserRepository
	if cfg.Enable {
		userRepo = keycloak.NewUserRepository(cfg)
		slog.Info("successfully initialized user repository")
	} else {
		userRepo = NewUserDummyRepo()
	}

	return &shared.Repository{
		Auth: authRepo,
		User: userRepo,
	}
}
