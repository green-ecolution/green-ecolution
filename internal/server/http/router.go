package http

import (
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/swagger"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/evaluation"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/info"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/plugin"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/region"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/sensor"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/tree"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/treecluster"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/user"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/vehicle"
	wateringplan "github.com/green-ecolution/backend/internal/server/http/handler/v1/watering_plan"
	"github.com/green-ecolution/backend/internal/utils"
)

func (s *Server) root(router fiber.Router, authMiddlewares ...fiber.Handler) {
	router.Get("/", func(c *fiber.Ctx) error {
		return c.SendString("Hello, World!")
	})
	s.api(router, authMiddlewares...)
}

func (s *Server) api(router fiber.Router, authMiddlewares ...fiber.Handler) {
	router.Route("/api", func(router fiber.Router) { s.v1(router, authMiddlewares...) })
}

func (s *Server) v1(router fiber.Router, authMiddlewares ...fiber.Handler) {
	authMiddleware := utils.Map(authMiddlewares, func(m fiber.Handler) any { return m })

	app := router.Group("/v1")

	app.Get("/swagger/*", swagger.HandlerDefault)

	app.Route("/info", func(router fiber.Router) {
		router.Use(authMiddleware...)
		info.RegisterRoutes(router, s.services.InfoService)
	})

	app.Route("/cluster", func(router fiber.Router) {
		router.Use(authMiddleware...)
		treecluster.RegisterRoutes(router, s.services.TreeClusterService)
	})

	app.Route("/tree", func(router fiber.Router) {
		router.Use(authMiddleware...)
		tree.RegisterRoutes(router, s.services.TreeService)
	})

	app.Route("/sensor", func(router fiber.Router) {
		router.Use(authMiddleware...)
		sensor.RegisterRoutes(router, s.services.SensorService)
	})

	app.Route("/user", func(router fiber.Router) {
		user.RegisterPublicRoutes(router, s.services.AuthService)
		router.Use(authMiddleware...)
		user.RegisterRoutes(router, s.services.AuthService)
	})

	app.Route("/region", func(router fiber.Router) {
		router.Use(authMiddleware...)
		region.RegisterRoutes(router, s.services.RegionService)
	})

	app.Route("/vehicle", func(router fiber.Router) {
		router.Use(authMiddleware...)
		vehicle.RegisterRoutes(router, s.services.VehicleService)
	})

	app.Route("/watering-plan", func(router fiber.Router) {
		router.Use(authMiddleware...)
		wateringplan.RegisterRoutes(router, s.services.WateringPlanService)
	})

	app.Route("/evaluation", func(router fiber.Router) {
		router.Use(authMiddleware...)
		evaluation.RegisterRoutes(router, s.services.EvaluationService)
	})

	app.Route("/plugin", func(router fiber.Router) {
		plugin.RegisterRoutes(router, s.services.PluginService, authMiddlewares...)
	})
}
