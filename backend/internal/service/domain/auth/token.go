package auth

import (
	"context"
	"errors"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

func (s *AuthService) RetrospectToken(ctx context.Context, token string) (*domain.IntroSpectTokenResult, error) {
	log := logger.GetLogger(ctx)
	result, err := s.authRepository.RetrospectToken(ctx, token)
	if err != nil {
		log.Debug("failed to retrospect token", "token", token, "error", err)
		return nil, service.MapError(ctx, errors.Join(err, errors.New("failed to retrospect token")), service.ErrorLogAll)
	}

	return result, nil
}

func (s *AuthService) RefreshToken(ctx context.Context, token string) (*domain.ClientToken, error) {
	log := logger.GetLogger(ctx)
	result, err := s.authRepository.RefreshToken(ctx, token)
	if err != nil {
		log.Debug("failed to refresh token", "token", token, "error", err)
		return nil, service.MapError(ctx, errors.Join(err, errors.New("failed to refresh token")), service.ErrorLogAll)
	}

	return result, nil
}
