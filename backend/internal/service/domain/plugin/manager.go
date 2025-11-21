package plugin

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"maps"
	"slices"
	"sync"
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/worker"
)

type PluginManagerConfig struct {
	interval time.Duration
	timeout  time.Duration
}

type PluginManagerOption func(*PluginManagerConfig)

var defaultPluginManagerConfig = PluginManagerConfig{
	timeout:  5 * time.Minute,
	interval: 1 * time.Minute,
}

func WithTimeout(timeout time.Duration) PluginManagerOption {
	slog.Debug("use plugin manager with timeout", "timeout", timeout)
	return func(cfg *PluginManagerConfig) {
		cfg.timeout = timeout
	}
}

func WithInterval(interval time.Duration) PluginManagerOption {
	slog.Debug("use plugin manager with interval", "interval", interval)
	return func(cfg *PluginManagerConfig) {
		cfg.interval = interval
	}
}

type PluginManager struct {
	PluginManagerConfig
	plugins        map[string]entities.Plugin
	heartbeats     map[string]time.Time
	mutex          sync.RWMutex
	validator      *validator.Validate
	authRepository storage.AuthRepository
}

var _ service.PluginService = (*PluginManager)(nil)

func NewPluginManager(authRepo storage.AuthRepository, opts ...PluginManagerOption) *PluginManager {
	cfg := defaultPluginManagerConfig
	for _, opt := range opts {
		opt(&cfg)
	}

	return &PluginManager{
		plugins:             make(map[string]entities.Plugin),
		heartbeats:          make(map[string]time.Time),
		validator:           validator.New(),
		authRepository:      authRepo,
		PluginManagerConfig: cfg,
	}
}

func (p *PluginManager) Register(ctx context.Context, plugin *entities.Plugin) (*entities.ClientToken, error) {
	log := logger.GetLogger(ctx)
	if err := p.validator.Struct(plugin); err != nil {
		log.Debug("failed to validate plugin struct", "error", err, "raw_plugin", fmt.Sprintf("%+v", plugin))
		return nil, service.MapError(ctx, errors.Join(err, service.ErrValidation), service.ErrorLogValidation)
	}

	log.Info("register new plugin", "plugin", plugin.Slug)

	token, err := p.authRepository.GetAccessTokenFromClientCredentials(ctx, plugin.Auth.ClientID, plugin.Auth.ClientSecret)
	if err != nil {
		log.Debug("failed to login plugin with credantials", "error", err, "plugin_client_id", plugin.Auth.ClientID, "plugin_client_secret", "*******")
		return nil, service.MapError(ctx, errors.Join(err, errors.New("failed to login plugin with credantials")), service.ErrorLogAll)
	}

	p.mutex.Lock()
	defer p.mutex.Unlock()

	if _, ok := p.plugins[plugin.Slug]; ok {
		log.Warn("the plugin you are trying to register is already registered in the system.", "plugin_slug", plugin.Slug)
		return nil, service.ErrPluginRegistered
	}

	p.plugins[plugin.Slug] = *plugin
	p.heartbeats[plugin.Slug] = time.Now()

	return token, nil
}

func (p *PluginManager) RefreshToken(ctx context.Context, auth *entities.AuthPlugin, slug string) (*entities.ClientToken, error) {
	log := logger.GetLogger(ctx)
	log.Debug("refresh token for plugin", "plugin_slug", slug, "client_id", auth.ClientID, "client_secret", "**********")

	token, err := p.authRepository.GetAccessTokenFromClientCredentials(ctx, auth.ClientID, auth.ClientSecret)
	if err != nil {
		log.Debug("failed to refresh plugin token with credantials", "error", err, "plugin_client_id", auth.ClientID, "plugin_client_secret", "*******")
		return nil, service.MapError(ctx, errors.Join(err, errors.New("failed to refresh plugin token with credantials")), service.ErrorLogAll)
	}

	return token, err
}

func (p *PluginManager) Get(ctx context.Context, slug string) (entities.Plugin, error) {
	log := logger.GetLogger(ctx)
	p.mutex.RLock()
	defer p.mutex.RUnlock()
	plugin, ok := p.plugins[slug]
	if !ok {
		log.Error("the plugin is not registered in the system", "plugin_slug", slug)
		return plugin, service.ErrPluginNotRegistered
	}

	return plugin, nil
}

func (p *PluginManager) GetAll(_ context.Context) ([]entities.Plugin, []time.Time) {
	p.mutex.RLock()
	defer p.mutex.RUnlock()
	return slices.Collect(maps.Values(p.plugins)), slices.Collect(maps.Values(p.heartbeats))
}

func (p *PluginManager) HeartBeat(ctx context.Context, slug string) error {
	log := logger.GetLogger(ctx)
	if slug == "" {
		log.Debug("the provided plugin slug is empty")
		return service.NewError(service.BadRequest, "slug is empty")
	}

	p.mutex.Lock()
	defer p.mutex.Unlock()
	if _, ok := p.heartbeats[slug]; !ok {
		log.Error("the plugin is not registered in the system", "plugin_slug", slug)
		return service.ErrPluginNotRegistered
	}

	p.heartbeats[slug] = time.Now()
	return nil
}

func (p *PluginManager) Unregister(ctx context.Context, slug string) {
	log := logger.GetLogger(ctx)
	log.Info("unregister plugin", "plugin_slug", slug)
	p.mutex.Lock()
	defer p.mutex.Unlock()
	delete(p.plugins, slug)
	delete(p.heartbeats, slug)
}

func (p *PluginManager) checkHeartbeats() []string {
	slugsToDelete := make([]string, 0)

	p.mutex.RLock()
	defer p.mutex.RUnlock()

	for slug, heartbeat := range p.heartbeats {
		if time.Since(heartbeat) > p.timeout {
			slugsToDelete = append(slugsToDelete, slug)
		}
	}

	return slugsToDelete
}

func (p *PluginManager) batchUnregister(ctx context.Context, slugs []string) {
	log := logger.GetLogger(ctx)
	if len(slugs) == 0 {
		return
	}

	p.mutex.Lock()
	defer p.mutex.Unlock()
	for _, slug := range slugs {
		log.Info("unregister plugin due to timeout", "plugin", slug)
		delete(p.plugins, slug)
		delete(p.heartbeats, slug)
	}
}

func (p *PluginManager) StartCleanup(ctx context.Context) {
	scheduler := worker.NewScheduler(p.interval, worker.SchedulerFunc(func(ctx context.Context) error {
		slugsToDelete := p.checkHeartbeats()
		p.batchUnregister(ctx, slugsToDelete)

		return nil
	}))

	scheduler.Run(ctx)
}

func (p *PluginManager) Ready() bool {
	return true
}
