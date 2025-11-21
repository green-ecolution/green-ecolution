package plugin

import (
	"context"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

var _ service.PluginService = (*DummyPluginManager)(nil)

// DummyPluginManager is used to disable the plugin service
type DummyPluginManager struct{}

func NewDummyPluginManager() *DummyPluginManager {
	return &DummyPluginManager{}
}

func (s *DummyPluginManager) Register(_ context.Context, _ *entities.Plugin) (*entities.ClientToken, error) {
	return nil, service.NewError(service.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) RefreshToken(_ context.Context, _ *entities.AuthPlugin, _ string) (*entities.ClientToken, error) {
	return nil, service.NewError(service.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) Get(_ context.Context, _ string) (entities.Plugin, error) {
	return entities.Plugin{}, service.NewError(service.NotFound, "plugin support is disabled")
}

func (s *DummyPluginManager) GetAll(_ context.Context) ([]entities.Plugin, []time.Time) {
	return []entities.Plugin{}, []time.Time{}
}

func (s *DummyPluginManager) HeartBeat(_ context.Context, _ string) error {
	return service.NewError(service.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) Unregister(_ context.Context, _ string) {}

func (s *DummyPluginManager) StartCleanup(_ context.Context) {}

func (s *DummyPluginManager) Ready() bool {
	return true
}
