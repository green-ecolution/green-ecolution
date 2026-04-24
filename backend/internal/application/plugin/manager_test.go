package plugin

import (
	"context"
	"errors"
	"net/url"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/auth"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/plugin"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/_mock"
)

var (
	testPlugin = &plugin.Plugin{
		Slug:        "test-plugin",
		Name:        "Test Plugin",
		Path:        url.URL{Scheme: "http", Host: "localhost:8080"},
		Version:     "1.0.0",
		Description: "A test plugin",
		Auth: plugin.AuthPlugin{
			ClientID:     "test-client-id",
			ClientSecret: "test-client-secret",
		},
	}

	testPlugin2 = &plugin.Plugin{
		Slug:        "test-plugin-2",
		Name:        "Test Plugin 2",
		Path:        url.URL{Scheme: "http", Host: "localhost:9090"},
		Version:     "2.0.0",
		Description: "Another test plugin",
		Auth: plugin.AuthPlugin{
			ClientID:     "test-client-id-2",
			ClientSecret: "test-client-secret-2",
		},
	}

	testAuthPlugin = &plugin.AuthPlugin{
		ClientID:     "test-client-id",
		ClientSecret: "test-client-secret",
	}

	testClientToken = &auth.ClientToken{
		AccessToken:  "test-access-token",
		ExpiresIn:    3600,
		RefreshToken: "test-refresh-token",
		Expiry:       time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
		TokenType:    "Bearer",
	}
)

func TestPluginManager_Register(t *testing.T) {
	t.Run("should register plugin successfully", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testPlugin.Auth.ClientID,
			testPlugin.Auth.ClientSecret,
		).Return(testClientToken, nil)

		// when
		token, err := svc.Register(context.Background(), testPlugin)

		// then
		assert.NoError(t, err)
		assert.Equal(t, testClientToken, token)

		got, err := svc.Get(context.Background(), testPlugin.Slug)
		assert.NoError(t, err)
		assert.Equal(t, testPlugin.Slug, got.Slug)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return validation error when slug is empty", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		invalidPlugin := &plugin.Plugin{
			Slug: "",
			Name: "No Slug Plugin",
		}

		// when
		token, err := svc.Register(context.Background(), invalidPlugin)

		// then
		assert.Error(t, err)
		assert.Nil(t, token)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when plugin is already registered", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testPlugin.Auth.ClientID,
			testPlugin.Auth.ClientSecret,
		).Return(testClientToken, nil).Times(2)

		_, _ = svc.Register(context.Background(), testPlugin)

		// when
		token, err := svc.Register(context.Background(), testPlugin)

		// then
		assert.ErrorIs(t, err, ports.ErrPluginRegistered)
		assert.Nil(t, token)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when auth fails", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testPlugin.Auth.ClientID,
			testPlugin.Auth.ClientSecret,
		).Return(nil, errors.New("auth failed"))

		// when
		token, err := svc.Register(context.Background(), testPlugin)

		// then
		assert.Error(t, err)
		assert.Nil(t, token)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_Get(t *testing.T) {
	t.Run("should return plugin when registered", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testPlugin.Auth.ClientID,
			testPlugin.Auth.ClientSecret,
		).Return(testClientToken, nil)

		_, _ = svc.Register(context.Background(), testPlugin)

		// when
		got, err := svc.Get(context.Background(), testPlugin.Slug)

		// then
		assert.NoError(t, err)
		assert.Equal(t, testPlugin.Slug, got.Slug)
		assert.Equal(t, testPlugin.Name, got.Name)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when plugin is not registered", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		// when
		_, err := svc.Get(context.Background(), "non-existent")

		// then
		assert.ErrorIs(t, err, ports.ErrPluginNotRegistered)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_GetAll(t *testing.T) {
	t.Run("should return all registered plugins", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything, mock.Anything, mock.Anything,
		).Return(testClientToken, nil).Times(2)

		_, _ = svc.Register(context.Background(), testPlugin)
		_, _ = svc.Register(context.Background(), testPlugin2)

		// when
		plugins, heartbeats := svc.GetAll(context.Background())

		// then
		assert.Len(t, plugins, 2)
		assert.Len(t, heartbeats, 2)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return empty when no plugins registered", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		// when
		plugins, heartbeats := svc.GetAll(context.Background())

		// then
		assert.Empty(t, plugins)
		assert.Empty(t, heartbeats)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_HeartBeat(t *testing.T) {
	t.Run("should update heartbeat successfully", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testPlugin.Auth.ClientID,
			testPlugin.Auth.ClientSecret,
		).Return(testClientToken, nil)

		_, _ = svc.Register(context.Background(), testPlugin)

		// when
		err := svc.HeartBeat(context.Background(), testPlugin.Slug)

		// then
		assert.NoError(t, err)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when slug is empty", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		// when
		err := svc.HeartBeat(context.Background(), "")

		// then
		assert.Error(t, err)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when plugin is not registered", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		// when
		err := svc.HeartBeat(context.Background(), "non-existent")

		// then
		assert.ErrorIs(t, err, ports.ErrPluginNotRegistered)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_Unregister(t *testing.T) {
	t.Run("should remove a registered plugin", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testPlugin.Auth.ClientID,
			testPlugin.Auth.ClientSecret,
		).Return(testClientToken, nil)

		_, _ = svc.Register(context.Background(), testPlugin)

		// when
		svc.Unregister(context.Background(), testPlugin.Slug)

		// then
		_, err := svc.Get(context.Background(), testPlugin.Slug)
		assert.ErrorIs(t, err, ports.ErrPluginNotRegistered)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should be a no-op for non-existent plugin", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		// when / then (no panic)
		svc.Unregister(context.Background(), "non-existent")

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_RefreshToken(t *testing.T) {
	t.Run("should refresh token successfully", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testAuthPlugin.ClientID,
			testAuthPlugin.ClientSecret,
		).Return(testClientToken, nil)

		// when
		token, err := svc.RefreshToken(context.Background(), testAuthPlugin, "test-plugin")

		// then
		assert.NoError(t, err)
		assert.Equal(t, testClientToken, token)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when auth fails", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			testAuthPlugin.ClientID,
			testAuthPlugin.ClientSecret,
		).Return(nil, errors.New("auth failed"))

		// when
		token, err := svc.RefreshToken(context.Background(), testAuthPlugin, "test-plugin")

		// then
		assert.Error(t, err)
		assert.Nil(t, token)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_Cleanup(t *testing.T) {
	t.Run("should remove expired plugins", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo, WithTimeout(1*time.Millisecond))

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything, mock.Anything, mock.Anything,
		).Return(testClientToken, nil).Times(2)

		_, _ = svc.Register(context.Background(), testPlugin)
		_, _ = svc.Register(context.Background(), testPlugin2)

		// wait for timeout to expire
		time.Sleep(10 * time.Millisecond)

		// keep TestPlugin2 alive
		_ = svc.HeartBeat(context.Background(), testPlugin2.Slug)

		// manually trigger cleanup via StartCleanup with a cancelled context
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		svc.StartCleanup(ctx)

		// then
		_, err := svc.Get(context.Background(), testPlugin.Slug)
		assert.ErrorIs(t, err, ports.ErrPluginNotRegistered)

		got, err := svc.Get(context.Background(), testPlugin2.Slug)
		assert.NoError(t, err)
		assert.Equal(t, testPlugin2.Slug, got.Slug)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_Ready(t *testing.T) {
	t.Run("should return true", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := NewPluginManager(mockAuthRepo)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)

		mockAuthRepo.AssertExpectations(t)
	})
}
