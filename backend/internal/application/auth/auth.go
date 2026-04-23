package auth

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type AuthService struct {
	authRepository entities.AuthRepository
	userRepo       entities.UserRepository
	cfg            *config.IdentityAuthConfig
}

func NewAuthService(repo entities.AuthRepository, userRepo entities.UserRepository, cfg *config.IdentityAuthConfig) ports.AuthService {
	return &AuthService{
		authRepository: repo,
		userRepo:       userRepo,
		cfg:            cfg,
	}
}

func (s *AuthService) Ready() bool {
	return s.authRepository != nil
}
