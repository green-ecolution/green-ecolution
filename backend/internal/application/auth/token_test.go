package auth

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func TestRestrospectToken(t *testing.T) {
	t.Run("should return result when success", func(t *testing.T) {
		// given
		token := "token"
		identityConfig := &config.IdentityAuthConfig{}
		expected := &entities.IntroSpectTokenResult{
			Active:   utils.P(true),
			Exp:      utils.P(123),
			AuthTime: utils.P(123),
			Type:     utils.P("token"),
		}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		// when
		authRepo.EXPECT().RetrospectToken(rootCtx, token).Return(expected, nil)
		resp, err := svc.RetrospectToken(rootCtx, token)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expected, resp)
	})

	t.Run("should return error when failed to retrospect token", func(t *testing.T) {
		// given
		token := "token"
		identityConfig := &config.IdentityAuthConfig{}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		// when
		authRepo.EXPECT().RetrospectToken(rootCtx, token).Return(nil, errors.New(""))
		_, err := svc.RetrospectToken(rootCtx, token)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "failed to retrospect token: ")
	})
}
