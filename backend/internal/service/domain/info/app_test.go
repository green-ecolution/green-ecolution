package info

import (
	"context"
	"log/slog"
	"net"
	"net/url"
	"testing"
	"time"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/stretchr/testify/assert"
)

var rootCtx = context.WithValue(context.Background(), "logger", slog.Default())

func TestNewInfoService(t *testing.T) {
	repo := storageMock.NewMockInfoRepository(t)
	t.Run("should create a new service", func(t *testing.T) {
		svc := NewInfoService(repo)
		assert.NotNil(t, svc)
	})
}

func TestGetAppInfo(t *testing.T) {
	t.Run("should error if GetAppInfo return error", func(t *testing.T) {
		// given
		repo := storageMock.NewMockInfoRepository(t)
		svc := NewInfoService(repo)
		tests := map[error]service.ErrorCode{
			storage.ErrIPNotFound:            service.InternalError,
			storage.ErrIFacesNotFound:        service.InternalError,
			storage.ErrIFacesAddressNotFound: service.InternalError,
			storage.ErrHostnameNotFound:      service.InternalError,
		}

		for k, v := range tests {
			// when
			repo.EXPECT().
				GetAppInfo(rootCtx).
				Return(nil, k)
			appInfo, err := svc.GetAppInfo(rootCtx)

			// then
			assert.Nil(t, appInfo)

			assert.Error(t, err)
			var expectError service.Error
			assert.ErrorAs(t, err, &expectError)
			assert.Equal(t, v, expectError.Code)
		}
	})

	t.Run("should return app info", func(t *testing.T) {
		// given
		repo := storageMock.NewMockInfoRepository(t)
		svc := NewInfoService(repo)
		buildTime := time.Now()
		expectedAppInfo := domain.App{
			Version:   "1.0.0",
			GoVersion: "1.16",
			BuildTime: buildTime,
			Git: domain.Git{
				Commit: "123456",
				Branch: "main",
				Repository: &url.URL{
					Scheme: "https",
					Host:   "github.com",
					Path:   "/green-ecolution/green-space-management",
				},
			},
			Server: domain.Server{
				OS:       "linux",
				Arch:     "amd64",
				Hostname: "localhost",
				URL: &url.URL{
					Scheme: "http",
					Host:   "localhost:8080",
				},
				IP:        net.IPv4(127, 0, 0, 1),
				Port:      8080,
				Interface: "eth0",
				Uptime:    time.Hour,
			},
		}

		givenAppInfo := domain.App{
			Version:   "1.0.0",
			GoVersion: "1.16",
			BuildTime: buildTime,
			Git: domain.Git{
				Commit: "123456",
				Branch: "main",
				Repository: &url.URL{
					Scheme: "https",
					Host:   "github.com",
					Path:   "/green-ecolution/green-space-management",
				},
			},
			Server: domain.Server{
				OS:       "linux",
				Arch:     "amd64",
				Hostname: "localhost",
				URL: &url.URL{
					Scheme: "http",
					Host:   "localhost:8080",
				},
				IP:        net.IPv4(127, 0, 0, 1),
				Port:      8080,
				Interface: "eth0",
				Uptime:    time.Hour,
			},
		}

		// when
		repo.EXPECT().GetAppInfo(rootCtx).Return(&givenAppInfo, nil)
		appInfo, err := svc.GetAppInfo(rootCtx)

		// then
		assert.NoError(t, err)
		assert.EqualValues(t, expectedAppInfo, *appInfo)
	})
}

func TestReady(t *testing.T) {
	t.Run("should return true if the service is ready", func(t *testing.T) {
		// given
		repo := storageMock.NewMockInfoRepository(t)
		svc := NewInfoService(repo)

		// when
		ready := svc.Ready()

		// then
		assert.True(t, ready)
	})

	t.Run("should return false if the service is not ready", func(t *testing.T) {
		// given
		svc := NewInfoService(nil)

		// when
		ready := svc.Ready()

		// then
		assert.False(t, ready)
	})
}
