package http

import (
	"context"
	"errors"
	"fmt"
	"io/fs"
	"log/slog"
	"time"

	"github.com/green-ecolution/backend/internal/worker"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/backend/internal/service"
)

type HTTPError struct {
	Error  string `json:"error"`
	Code   int    `json:"code"`
	Path   string `json:"path"`
	Method string `json:"method"`
} //	@Name	HTTPError

type Server struct {
	cfg        *config.Config
	frontendFs fs.FS
	services   *service.Services
}

func NewServer(cfg *config.Config, services *service.Services, frontendFs fs.FS) *Server {
	return &Server{
		cfg:        cfg,
		frontendFs: frontendFs,
		services:   services,
	}
}

func (s *Server) Run(ctx context.Context) error {
	app := fiber.New(fiber.Config{
		AppName:                  s.cfg.Dashboard.Title,
		ServerHeader:             s.cfg.Dashboard.Title,
		ErrorHandler:             errorHandler,
		EnableSplittingOnParsers: true,
	})

	app.Mount("/", s.middleware())

	go func() {
		slog.Info("starting plugin cleanup service: cleaning up unhealthy plugins")
		s.services.PluginService.StartCleanup(ctx)
	}()

	sensorStatusScheduler := worker.NewScheduler(3*time.Hour, worker.SchedulerFunc(s.services.SensorService.UpdateStatuses))
	go sensorStatusScheduler.Run(ctx)

	var onceADay = 24 * time.Hour

	wateringPlanStatusScheduler := worker.NewScheduler(onceADay, worker.SchedulerFunc(s.services.WateringPlanService.UpdateStatuses))
	go wateringPlanStatusScheduler.Run(ctx)

	clusterWateringStatusScheduler := worker.NewScheduler(onceADay, worker.SchedulerFunc(s.services.TreeClusterService.UpdateWateringStatuses))
	go clusterWateringStatusScheduler.Run(ctx)

	treeWateringStatusScheduler := worker.NewScheduler(onceADay, worker.SchedulerFunc(s.services.TreeService.UpdateWateringStatuses))
	go treeWateringStatusScheduler.Run(ctx)

	go func() {
		<-ctx.Done()
		slog.Info("shutting down http server")
		if err := app.Shutdown(); err != nil {
			slog.Error("error while shutting down http server", "error", err, "service", "fiber")
		}
	}()

	slog.Info("starting server", "url", s.cfg.Server.AppURL, "port", s.cfg.Server.Port, "service", "fiber")
	return app.Listen(fmt.Sprintf(":%d", s.cfg.Server.Port))
}

func errorHandler(c *fiber.Ctx, err error) error {
	code := fiber.StatusInternalServerError

	var e *fiber.Error
	if errors.As(err, &e) {
		code = e.Code
	}

	return c.Status(code).JSON(HTTPError{
		Error:  err.Error(),
		Code:   code,
		Path:   c.Path(),
		Method: c.Method(),
	})
}
