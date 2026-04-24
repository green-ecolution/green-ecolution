package plugin

import (
	"context"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/auth"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/plugin"
)

var _ ports.PluginService = (*DummyPluginManager)(nil)

// DummyPluginManager is used to disable the plugin service
type DummyPluginManager struct{}

func NewDummyPluginManager() *DummyPluginManager {
	return &DummyPluginManager{}
}

func (s *DummyPluginManager) Register(_ context.Context, _ *plugin.Plugin) (*auth.ClientToken, error) {
	return nil, ports.NewError(ports.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) RefreshToken(_ context.Context, _ *plugin.AuthPlugin, _ string) (*auth.ClientToken, error) {
	return nil, ports.NewError(ports.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) Get(_ context.Context, _ string) (plugin.Plugin, error) {
	return plugin.Plugin{}, ports.NewError(ports.NotFound, "plugin support is disabled")
}

func (s *DummyPluginManager) GetAll(_ context.Context) ([]plugin.Plugin, []time.Time) {
	return []plugin.Plugin{}, []time.Time{}
}

func (s *DummyPluginManager) HeartBeat(_ context.Context, _ string) error {
	return ports.NewError(ports.Gone, "plugin support is disabled")
}

func (s *DummyPluginManager) Unregister(_ context.Context, _ string) {}

func (s *DummyPluginManager) StartCleanup(_ context.Context) {}

func (s *DummyPluginManager) Ready() bool {
	return true
}
