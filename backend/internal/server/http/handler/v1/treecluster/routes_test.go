package treecluster_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"testing"

	"github.com/green-ecolution/backend/internal/entities"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/treecluster"
	"github.com/green-ecolution/backend/internal/server/http/middleware"
	serviceMock "github.com/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterRoutes(t *testing.T) {
	t.Run("/v1/cluster", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockClusterService := serviceMock.NewMockTreeClusterService(t)
			app := fiber.New()
			app.Use(middleware.PaginationMiddleware())
			treecluster.RegisterRoutes(app, mockClusterService)

			ctx := context.WithValue(context.Background(), "page", int32(1))
			ctx = context.WithValue(ctx, "limit", int32(-1))

			mockClusterService.EXPECT().GetAll(
				mock.Anything, entities.TreeClusterQuery{},
			).Return(TestClusterList, int64(len(TestClusterList)), nil)

			// when
			req, _ := http.NewRequestWithContext(ctx, http.MethodGet, "/", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call POST handler", func(t *testing.T) {
			mockClusterService := serviceMock.NewMockTreeClusterService(t)
			app := fiber.New()
			treecluster.RegisterRoutes(app, mockClusterService)

			mockClusterService.EXPECT().Create(
				mock.Anything,
				mock.AnythingOfType("*entities.TreeClusterCreate"),
			).Return(TestCluster, nil)

			// when
			body, _ := json.Marshal(TestClusterRequest)
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusCreated, resp.StatusCode)
		})
	})

	t.Run("/v1/cluster/:id", func(t *testing.T) {
		t.Run("should call GET handler", func(t *testing.T) {
			mockClusterService := serviceMock.NewMockTreeClusterService(t)
			app := fiber.New()
			treecluster.RegisterRoutes(app, mockClusterService)

			mockClusterService.EXPECT().GetByID(
				mock.Anything,
				int32(1),
			).Return(TestCluster, nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call PUT handler", func(t *testing.T) {
			mockClusterService := serviceMock.NewMockTreeClusterService(t)
			app := fiber.New()
			treecluster.RegisterRoutes(app, mockClusterService)

			mockClusterService.EXPECT().Update(
				mock.Anything,
				int32(1),
				mock.Anything,
			).Return(TestCluster, nil)

			// when
			body, _ := json.Marshal(TestClusterRequest)
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodPut, "/1", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, resp.StatusCode)
		})

		t.Run("should call DELETE handler", func(t *testing.T) {
			mockClusterService := serviceMock.NewMockTreeClusterService(t)
			app := fiber.New()
			treecluster.RegisterRoutes(app, mockClusterService)

			mockClusterService.EXPECT().Delete(
				mock.Anything,
				int32(1),
			).Return(nil)

			// when
			req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/1", nil)

			// then
			resp, err := app.Test(req)
			defer resp.Body.Close()
			assert.NoError(t, err)
			assert.Equal(t, http.StatusNoContent, resp.StatusCode)
		})
	})
}
