package auth

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type AuthService struct {
	authRepository shared.AuthRepository
	userRepo       shared.UserRepository
	cfg            *config.IdentityAuthConfig
}

func NewAuthService(repo shared.AuthRepository, userRepo shared.UserRepository, cfg *config.IdentityAuthConfig) ports.AuthService {
	return &AuthService{
		authRepository: repo,
		userRepo:       userRepo,
		cfg:            cfg,
	}
}

func (s *AuthService) Ready() bool {
	return s.authRepository != nil
}
