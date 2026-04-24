package auth

import (
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/user"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/auth/keycloak"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
)

func NewRepository(cfg *config.IdentityAuthConfig) *storage.Repository {
	authRepo := keycloak.NewKeycloakRepository(cfg)
	slog.Info("successfully initialized auth repository", "service", "keycloak")

	var userRepo user.UserRepository
	if cfg.Enable {
		userRepo = keycloak.NewUserRepository(cfg)
		slog.Info("successfully initialized user repository")
	} else {
		userRepo = NewUserDummyRepo()
	}

	return &storage.Repository{
		Auth: authRepo,
		User: userRepo,
	}
}
