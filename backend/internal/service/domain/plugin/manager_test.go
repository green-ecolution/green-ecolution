package plugin_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/service/domain/plugin"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestPluginManager_Register(t *testing.T) {
	t.Run("should register plugin successfully", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestPlugin.Auth.ClientID,
			TestPlugin.Auth.ClientSecret,
		).Return(TestClientToken, nil)

		// when
		token, err := svc.Register(context.Background(), TestPlugin)

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestClientToken, token)

		got, err := svc.Get(context.Background(), TestPlugin.Slug)
		assert.NoError(t, err)
		assert.Equal(t, TestPlugin.Slug, got.Slug)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return validation error when slug is empty", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		invalidPlugin := &entities.Plugin{
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
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestPlugin.Auth.ClientID,
			TestPlugin.Auth.ClientSecret,
		).Return(TestClientToken, nil).Times(2)

		_, _ = svc.Register(context.Background(), TestPlugin)

		// when
		token, err := svc.Register(context.Background(), TestPlugin)

		// then
		assert.ErrorIs(t, err, service.ErrPluginRegistered)
		assert.Nil(t, token)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when auth fails", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestPlugin.Auth.ClientID,
			TestPlugin.Auth.ClientSecret,
		).Return(nil, errors.New("auth failed"))

		// when
		token, err := svc.Register(context.Background(), TestPlugin)

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
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestPlugin.Auth.ClientID,
			TestPlugin.Auth.ClientSecret,
		).Return(TestClientToken, nil)

		_, _ = svc.Register(context.Background(), TestPlugin)

		// when
		got, err := svc.Get(context.Background(), TestPlugin.Slug)

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestPlugin.Slug, got.Slug)
		assert.Equal(t, TestPlugin.Name, got.Name)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when plugin is not registered", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		// when
		_, err := svc.Get(context.Background(), "non-existent")

		// then
		assert.ErrorIs(t, err, service.ErrPluginNotRegistered)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_GetAll(t *testing.T) {
	t.Run("should return all registered plugins", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything, mock.Anything, mock.Anything,
		).Return(TestClientToken, nil).Times(2)

		_, _ = svc.Register(context.Background(), TestPlugin)
		_, _ = svc.Register(context.Background(), TestPlugin2)

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
		svc := plugin.NewPluginManager(mockAuthRepo)

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
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestPlugin.Auth.ClientID,
			TestPlugin.Auth.ClientSecret,
		).Return(TestClientToken, nil)

		_, _ = svc.Register(context.Background(), TestPlugin)

		// when
		err := svc.HeartBeat(context.Background(), TestPlugin.Slug)

		// then
		assert.NoError(t, err)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when slug is empty", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		// when
		err := svc.HeartBeat(context.Background(), "")

		// then
		assert.Error(t, err)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when plugin is not registered", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		// when
		err := svc.HeartBeat(context.Background(), "non-existent")

		// then
		assert.ErrorIs(t, err, service.ErrPluginNotRegistered)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_Unregister(t *testing.T) {
	t.Run("should remove a registered plugin", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestPlugin.Auth.ClientID,
			TestPlugin.Auth.ClientSecret,
		).Return(TestClientToken, nil)

		_, _ = svc.Register(context.Background(), TestPlugin)

		// when
		svc.Unregister(context.Background(), TestPlugin.Slug)

		// then
		_, err := svc.Get(context.Background(), TestPlugin.Slug)
		assert.ErrorIs(t, err, service.ErrPluginNotRegistered)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should be a no-op for non-existent plugin", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		// when / then (no panic)
		svc.Unregister(context.Background(), "non-existent")

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_RefreshToken(t *testing.T) {
	t.Run("should refresh token successfully", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestAuthPlugin.ClientID,
			TestAuthPlugin.ClientSecret,
		).Return(TestClientToken, nil)

		// when
		token, err := svc.RefreshToken(context.Background(), TestAuthPlugin, "test-plugin")

		// then
		assert.NoError(t, err)
		assert.Equal(t, TestClientToken, token)

		mockAuthRepo.AssertExpectations(t)
	})

	t.Run("should return error when auth fails", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything,
			TestAuthPlugin.ClientID,
			TestAuthPlugin.ClientSecret,
		).Return(nil, errors.New("auth failed"))

		// when
		token, err := svc.RefreshToken(context.Background(), TestAuthPlugin, "test-plugin")

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
		svc := plugin.NewPluginManager(mockAuthRepo, plugin.WithTimeout(1*time.Millisecond))

		mockAuthRepo.EXPECT().GetAccessTokenFromClientCredentials(
			mock.Anything, mock.Anything, mock.Anything,
		).Return(TestClientToken, nil).Times(2)

		_, _ = svc.Register(context.Background(), TestPlugin)
		_, _ = svc.Register(context.Background(), TestPlugin2)

		// wait for timeout to expire
		time.Sleep(10 * time.Millisecond)

		// keep TestPlugin2 alive
		_ = svc.HeartBeat(context.Background(), TestPlugin2.Slug)

		// manually trigger cleanup via StartCleanup with a cancelled context
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		svc.StartCleanup(ctx)

		// then
		_, err := svc.Get(context.Background(), TestPlugin.Slug)
		assert.ErrorIs(t, err, service.ErrPluginNotRegistered)

		got, err := svc.Get(context.Background(), TestPlugin2.Slug)
		assert.NoError(t, err)
		assert.Equal(t, TestPlugin2.Slug, got.Slug)

		mockAuthRepo.AssertExpectations(t)
	})
}

func TestPluginManager_Ready(t *testing.T) {
	t.Run("should return true", func(t *testing.T) {
		// given
		mockAuthRepo := storageMock.NewMockAuthRepository(t)
		svc := plugin.NewPluginManager(mockAuthRepo)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)

		mockAuthRepo.AssertExpectations(t)
	})
}
