package plugin_test

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"testing"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/plugin"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestRegisterPlugin(t *testing.T) {
	t.Run("should register plugin successfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.RegisterPlugin(mockPluginService)
		app.Post("/v1/plugin/register", handler)

		mockPluginService.EXPECT().Register(
			mock.Anything,
			mock.AnythingOfType("*entities.Plugin"),
		).Return(TestClientToken, nil)

		reqBody := serverEntities.PluginRegisterRequest{
			Slug:        TestPlugin.Slug,
			Name:        TestPlugin.Name,
			Description: TestPlugin.Description,
			Version:     TestPlugin.Version,
			Path:        TestPlugin.Path.String(),
			Auth: serverEntities.PluginAuth{
				ClientID:     TestPlugin.Auth.ClientID,
				ClientSecret: TestPlugin.Auth.ClientSecret,
			},
		}

		// when
		body, _ := json.Marshal(reqBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/register", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.ClientTokenResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, TestClientToken.AccessToken, response.AccessToken)
		assert.Equal(t, TestClientToken.TokenType, response.TokenType)

		mockPluginService.AssertExpectations(t)
	})

	t.Run("should return 400 when auth credentials are empty", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.RegisterPlugin(mockPluginService)
		app.Post("/v1/plugin/register", handler)

		reqBody := serverEntities.PluginRegisterRequest{
			Slug: TestPlugin.Slug,
			Name: TestPlugin.Name,
			Auth: serverEntities.PluginAuth{
				ClientID:     "",
				ClientSecret: "",
			},
		}

		// when
		body, _ := json.Marshal(reqBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/register", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})

	t.Run("should return 400 when service returns error", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.RegisterPlugin(mockPluginService)
		app.Post("/v1/plugin/register", handler)

		mockPluginService.EXPECT().Register(
			mock.Anything,
			mock.AnythingOfType("*entities.Plugin"),
		).Return(nil, errors.New("registration failed"))

		reqBody := serverEntities.PluginRegisterRequest{
			Slug: TestPlugin.Slug,
			Name: TestPlugin.Name,
			Path: TestPlugin.Path.String(),
			Auth: serverEntities.PluginAuth{
				ClientID:     TestPlugin.Auth.ClientID,
				ClientSecret: TestPlugin.Auth.ClientSecret,
			},
		}

		// when
		body, _ := json.Marshal(reqBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/register", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})
}

func TestGetPluginsList(t *testing.T) {
	t.Run("should return all plugins successfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.GetPluginsList(mockPluginService)
		app.Get("/v1/plugin", handler)

		heartbeats := []time.Time{time.Now(), time.Now()}
		mockPluginService.EXPECT().GetAll(
			mock.Anything,
		).Return(TestPlugins, heartbeats)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/plugin", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.PluginListResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Len(t, response.Plugins, 2)
		assert.Equal(t, TestPlugins[0].Slug, response.Plugins[0].Slug)

		mockPluginService.AssertExpectations(t)
	})

	t.Run("should return empty list when no plugins registered", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.GetPluginsList(mockPluginService)
		app.Get("/v1/plugin", handler)

		mockPluginService.EXPECT().GetAll(
			mock.Anything,
		).Return([]entities.Plugin{}, []time.Time{})

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/plugin", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.PluginListResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Empty(t, response.Plugins)

		mockPluginService.AssertExpectations(t)
	})
}

func TestGetPluginInfo(t *testing.T) {
	t.Run("should return plugin info successfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.GetPluginInfo(mockPluginService)
		app.Get("/v1/plugin/:plugin", handler)

		mockPluginService.EXPECT().Get(
			mock.Anything,
			TestPlugin.Slug,
		).Return(TestPlugin, nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/plugin/"+TestPlugin.Slug, nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.PluginResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, TestPlugin.Slug, response.Slug)
		assert.Equal(t, TestPlugin.Name, response.Name)

		mockPluginService.AssertExpectations(t)
	})

	t.Run("should return 404 when plugin is not found", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.GetPluginInfo(mockPluginService)
		app.Get("/v1/plugin/:plugin", handler)

		mockPluginService.EXPECT().Get(
			mock.Anything,
			"non-existent",
		).Return(entities.Plugin{}, errors.New("not found"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/plugin/non-existent", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})
}

func TestPluginHeartbeat(t *testing.T) {
	t.Run("should return 200 on successful heartbeat", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.PluginHeartbeat(mockPluginService)
		app.Post("/v1/plugin/:plugin/heartbeat", handler)

		mockPluginService.EXPECT().HeartBeat(
			mock.Anything,
			TestPlugin.Slug,
		).Return(nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/"+TestPlugin.Slug+"/heartbeat", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})

	t.Run("should return 400 when service returns error", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.PluginHeartbeat(mockPluginService)
		app.Post("/v1/plugin/:plugin/heartbeat", handler)

		mockPluginService.EXPECT().HeartBeat(
			mock.Anything,
			"non-existent",
		).Return(errors.New("plugin not registered"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/non-existent/heartbeat", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})
}

func TestUnregisterPlugin(t *testing.T) {
	t.Run("should return 204 on successful unregister", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.UnregisterPlugin(mockPluginService)
		app.Post("/v1/plugin/:plugin/unregister", handler)

		mockPluginService.EXPECT().Unregister(
			mock.Anything,
			TestPlugin.Slug,
		).Return()

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/"+TestPlugin.Slug+"/unregister", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNoContent, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})
}

func TestRefreshToken(t *testing.T) {
	t.Run("should refresh token successfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.RefreshToken(mockPluginService)
		app.Post("/v1/plugin/:plugin/token/refresh", handler)

		mockPluginService.EXPECT().RefreshToken(
			mock.Anything,
			mock.AnythingOfType("*entities.AuthPlugin"),
			TestPlugin.Slug,
		).Return(TestClientToken, nil)

		reqBody := serverEntities.PluginAuth{
			ClientID:     TestPlugin.Auth.ClientID,
			ClientSecret: TestPlugin.Auth.ClientSecret,
		}

		// when
		body, _ := json.Marshal(reqBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/"+TestPlugin.Slug+"/token/refresh", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.ClientTokenResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.NoError(t, err)
		assert.Equal(t, TestClientToken.AccessToken, response.AccessToken)

		mockPluginService.AssertExpectations(t)
	})

	t.Run("should return 400 when credentials are empty", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.RefreshToken(mockPluginService)
		app.Post("/v1/plugin/:plugin/token/refresh", handler)

		reqBody := serverEntities.PluginAuth{
			ClientID:     "",
			ClientSecret: "",
		}

		// when
		body, _ := json.Marshal(reqBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/"+TestPlugin.Slug+"/token/refresh", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})

	t.Run("should return error when service fails", func(t *testing.T) {
		// given
		app := fiber.New()
		mockPluginService := serviceMock.NewMockPluginService(t)
		handler := plugin.RefreshToken(mockPluginService)
		app.Post("/v1/plugin/:plugin/token/refresh", handler)

		mockPluginService.EXPECT().RefreshToken(
			mock.Anything,
			mock.AnythingOfType("*entities.AuthPlugin"),
			TestPlugin.Slug,
		).Return(nil, errors.New("auth failed"))

		reqBody := serverEntities.PluginAuth{
			ClientID:     TestPlugin.Auth.ClientID,
			ClientSecret: TestPlugin.Auth.ClientSecret,
		}

		// when
		body, _ := json.Marshal(reqBody)
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodPost, "/v1/plugin/"+TestPlugin.Slug+"/token/refresh", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockPluginService.AssertExpectations(t)
	})
}
