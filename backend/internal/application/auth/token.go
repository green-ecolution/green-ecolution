package auth

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/auth"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

func (s *AuthService) RetrospectToken(ctx context.Context, token string) (*auth.IntroSpectTokenResult, error) {
	log := logger.GetLogger(ctx)
	result, err := s.authRepository.RetrospectToken(ctx, token)
	if err != nil {
		log.Debug("failed to retrospect token", "token", token, "error", err)
		return nil, ports.MapError(ctx, errors.Join(err, errors.New("failed to retrospect token")), ports.ErrorLogAll)
	}

	return result, nil
}

func (s *AuthService) RefreshToken(ctx context.Context, token string) (*auth.ClientToken, error) {
	log := logger.GetLogger(ctx)
	result, err := s.authRepository.RefreshToken(ctx, token)
	if err != nil {
		log.Debug("failed to refresh token", "token", token, "error", err)
		return nil, ports.MapError(ctx, errors.Join(err, errors.New("failed to refresh token")), ports.ErrorLogAll)
	}

	return result, nil
}
