package auth

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/auth"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/user"
)

type AuthService struct {
	authRepository auth.AuthRepository
	userRepo       user.UserRepository
	cfg            *config.IdentityAuthConfig
}

func NewAuthService(repo auth.AuthRepository, userRepo user.UserRepository, cfg *config.IdentityAuthConfig) ports.AuthService {
	return &AuthService{
		authRepository: repo,
		userRepo:       userRepo,
		cfg:            cfg,
	}
}

func (s *AuthService) Ready() bool {
	return s.authRepository != nil
}
