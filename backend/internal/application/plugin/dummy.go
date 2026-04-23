package plugin

import (
	"context"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

var _ ports.PluginService = (*DummyPluginManager)(nil)

// DummyPluginManager is used to disable the plugin service
type DummyPluginManager struct{}

func NewDummyPluginManager() *DummyPluginManager {
	return &DummyPluginManager{}
}

func (s *DummyPluginManager) Register(_ context.Context, _ *entities.Plugin) (*entities.ClientToken, error) {
	return nil, ports.NewError(ports.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) RefreshToken(_ context.Context, _ *entities.AuthPlugin, _ string) (*entities.ClientToken, error) {
	return nil, ports.NewError(ports.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) Get(_ context.Context, _ string) (entities.Plugin, error) {
	return entities.Plugin{}, ports.NewError(ports.NotFound, "plugin support is disabled")
}

func (s *DummyPluginManager) GetAll(_ context.Context) ([]entities.Plugin, []time.Time) {
	return []entities.Plugin{}, []time.Time{}
}

func (s *DummyPluginManager) HeartBeat(_ context.Context, _ string) error {
	return ports.NewError(ports.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) Unregister(_ context.Context, _ string) {}

func (s *DummyPluginManager) StartCleanup(_ context.Context) {}

func (s *DummyPluginManager) Ready() bool {
	return true
}
