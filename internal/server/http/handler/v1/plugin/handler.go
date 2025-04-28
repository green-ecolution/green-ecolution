package plugin

import (
	"log"
	"log/slog"
	"net/http/httputil"
	"net/url"
	"strings"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/adaptor"
	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/utils"
)

func getPluginFiles(svc service.PluginService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		pluginParam := strings.Clone(c.Params("plugin"))
		plugin, err := svc.Get(ctx, pluginParam)
		if err != nil {
			return err
		}

		reverseProxy := httputil.ReverseProxy{
			Rewrite: func(r *httputil.ProxyRequest) {
				r.SetURL(&plugin.Path)
				r.Out.Host = r.In.Host
				r.Out.URL.Path = strings.Replace(r.In.URL.Path, "/api/v1/plugin/"+plugin.Slug, plugin.Path.String(), 1)
				r.SetXForwarded()
			},
		}

		return adaptor.HTTPHandler(&reverseProxy)(c)
	}
}

// @Summary		Register a plugin
// @Description	Register a plugin
// @Id				register-plugin
// @Tags			Plugin
// @Produce		json
// @Success		200	{object}	entities.ClientTokenResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/plugin [post]
//
// @Param			body	body	entities.PluginRegisterRequest	true	"Plugin registration request"
func RegisterPlugin(svc service.PluginService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		var req entities.PluginRegisterRequest
		if err := c.BodyParser(&req); err != nil {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": err.Error(),
			})
		}

		if req.Auth.ClientID == "" || req.Auth.ClientSecret == "" {
			return c.Status(fiber.StatusBadRequest).SendString("Username or password is empty")
		}

		path, err := url.Parse(req.Path)
		if err != nil {
			slog.Error("Failed to parse plugin path", "error", err)
			return c.Status(fiber.StatusBadRequest).SendString("Failed to parse plugin path")
		}

		plugin := &domain.Plugin{
			Name:        req.Name,
			Path:        *path,
			Version:     req.Version,
			Description: req.Description,
			Slug:        req.Slug,
			Auth: domain.AuthPlugin{
				ClientID:     req.Auth.ClientID,
				ClientSecret: req.Auth.ClientSecret,
			},
		}

		token, err := svc.Register(c.Context(), plugin)
		if err != nil {
			slog.Error("Failed to register plugin", "error", err)
			return c.Status(fiber.StatusBadRequest).SendString(err.Error())
		}

		response := entities.ClientTokenResponse{
			AccessToken:  token.AccessToken,
			ExpiresIn:    token.ExpiresIn,
			RefreshToken: token.RefreshToken,
			Expiry:       token.Expiry,
			TokenType:    token.TokenType,
		}

		return c.Status(fiber.StatusOK).JSON(response)
	}
}

// @Summary		Unregister a plugin
// @Description	Unregister a plugin
// @Id				unregister-plugin
// @Tags			Plugin
// @Produce		json
// @Success		204
// @Failure		401	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/plugin/{plugin_slug}/unregister [post]
// @Param			plugin_slug	path	string	true	"Slug of the plugin"
// @Security		Keycloak
func UnregisterPlugin(svc service.PluginService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		slug := strings.Clone(c.Params("plugin"))
		svc.Unregister(ctx, slug)

		return c.SendStatus(fiber.StatusNoContent)
	}
}

// @Summary		Heartbeat for a plugin
// @Description	Heartbeat for a plugin
// @Id				plugin-heartbeat
// @Tags			Plugin
// @Produce		json
// @Success		200	{object}	string
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/plugin/{plugin_slug}/heartbeat [post]
// @Param			plugin_slug	path	string	true	"Name of the plugin specified by slug during registration"
// @Security		Keycloak
func PluginHeartbeat(svc service.PluginService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		slug := strings.Clone(c.Params("plugin"))
		if err := svc.HeartBeat(ctx, slug); err != nil {
			slog.Error("Failed to heartbeat", "plugin", slug, "error", err)
			return c.Status(fiber.StatusBadRequest).SendString(err.Error())
		}

		return c.Status(fiber.StatusOK).SendString("Heartbeat received")
	}
}

// @Summary		Get a list of all registered plugins
// @Description	Get a list of all registered plugins
// @Id				get-plugins-list
// @Tags			Plugin
// @Produce		json
// @Success		200	{object}	entities.PluginListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/plugin [get]
// @Security		Keycloak
func GetPluginsList(svc service.PluginService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		p, h := svc.GetAll(ctx)
		log.Println(h)
		plugins := utils.Map(p, func(plugin domain.Plugin) entities.PluginResponse {
			return entities.PluginResponse{
				Slug:        plugin.Slug,
				Name:        plugin.Name,
				Version:     plugin.Version,
				Description: plugin.Description,
				HostPath:    plugin.Path.String(),
			}
		})

		return c.Status(fiber.StatusOK).JSON(entities.PluginListResponse{
			Plugins: plugins,
		})
	}
}

// @Summary		Get a plugin info
// @Description	Get a plugin info
// @Id				get-plugin-info
// @Tags			Plugin
// @Produce		json
// @Success		200	{object}	entities.PluginResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		404	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/plugin/{plugin_slug} [get]
// @Param			plugin_slug	path	string	true	"Slug of the plugin"
// @Security		Keycloak
func GetPluginInfo(svc service.PluginService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		pluginParam := strings.Clone(c.Params("plugin"))
		plugin, err := svc.Get(ctx, pluginParam)
		if err != nil {
			return c.Status(fiber.StatusNotFound).SendString("plugin not found")
		}

		return c.Status(fiber.StatusOK).JSON(entities.PluginResponse{
			Slug:        plugin.Slug,
			Name:        plugin.Name,
			Version:     plugin.Version,
			Description: plugin.Description,
			HostPath:    plugin.Path.String(),
		})
	}
}

// @Summary		Refresh plugin token
// @Description	Refresh plugin token
// @Id				refresh-plugin-token
// @Tags			Plugin
// @Accept			json
// @Produce		json
// @Param			body	body		entities.PluginAuth	true	"Plugin authentication"
// @Success		200		{object}	entities.ClientTokenResponse
// @Failure		400		{object}	HTTPError
// @Failure		500		{object}	HTTPError
// @Router			/v1/plugin/{plugin_slug}/token/refresh [post]
// @Param			plugin_slug	path	string	true	"Slug of the plugin"
func RefreshToken(svc service.PluginService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		var req entities.PluginAuth
		if err := c.BodyParser(&req); err != nil {
			return errorhandler.HandleError(err)
		}

		if req.ClientID == "" || req.ClientSecret == "" {
			return c.Status(fiber.StatusBadRequest).SendString("client_id or client_secret is empty")
		}

		auth := domain.AuthPlugin{
			ClientID:     req.ClientID,
			ClientSecret: req.ClientSecret,
		}
		pluginSlug := strings.Clone(c.Params("plugin"))

		token, err := svc.RefreshToken(c.Context(), &auth, pluginSlug)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		response := entities.ClientTokenResponse{
			AccessToken:  token.AccessToken,
			ExpiresIn:    token.ExpiresIn,
			RefreshToken: token.RefreshToken,
			Expiry:       token.Expiry,
			TokenType:    token.TokenType,
		}

		return c.Status(fiber.StatusOK).JSON(response)
	}
}
