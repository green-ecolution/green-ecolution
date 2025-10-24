package plugin

import "net/url"

// Plugin represents a plugin that can be registered with the plugin host.
//
// Fields:
// - Slug: A unique identifier for the plugin, used to distinguish it within the plugin host.
// - Name: The name of the plugin, used for display and identification purposes.
// - Version: The version of the plugin, typically following semantic versioning (e.g., "1.0.0").
// - Description: A brief description of the plugin, detailing its purpose or functionality.
// - PluginHostPath: The URL path where the plugin can be accessed on the plugin host.
type Plugin struct {
	Slug           string
	Name           string
	Version        string
	Description    string
	PluginHostPath *url.URL
}

// PluginOption is a functional option for configuring a Plugin.
//
// Functional options provide a flexible and extensible way to customize Plugin instances
// during creation without requiring multiple constructors or complex initialization logic.
type PluginOption func(*Plugin)

// WithName sets the name of the plugin.
//
// Example usage:
//
//	plugin := NewPlugin(WithName("My Plugin"))
func WithName(name string) PluginOption {
	return func(p *Plugin) {
		p.Name = name
	}
}

// WithSlug sets the slug of the plugin.
//
// Example usage:
//
//	plugin := NewPlugin(WithSlug("my-plugin"))
func WithSlug(slug string) PluginOption {
	return func(p *Plugin) {
		p.Slug = slug
	}
}

// WithVersion sets the version of the plugin.
//
// Example usage:
//
//	plugin := NewPlugin(WithVersion("1.0.0"))
func WithVersion(version string) PluginOption {
	return func(p *Plugin) {
		p.Version = version
	}
}

// WithDescription sets the description of the plugin.
//
// Example usage:
//
//	plugin := NewPlugin(WithDescription("This plugin provides example functionality."))
func WithDescription(description string) PluginOption {
	return func(p *Plugin) {
		p.Description = description
	}
}

// WithHostPath sets the host path of the plugin.
//
// Example usage:
//
//	hostPath, _ := url.Parse("https://example.com/plugin")
//	plugin := NewPlugin(WithHostPath(hostPath))
func WithHostPath(hostPath *url.URL) PluginOption {
	return func(p *Plugin) {
		p.PluginHostPath = hostPath
	}
}

// defaultPlugin provides default values for Plugin instances.
// By default, the version is set to "develop". Other fields must be explicitly configured via options.
var defaultPlugin = Plugin{
	Version: "develop",
}

// NewPlugin creates a new Plugin instance with the provided functional options.
//
// If no options are provided, the defaultPlugin is used as the base configuration.
// Each option is applied sequentially to modify the default settings.
//
// Parameters:
// - opts: A variadic list of PluginOption functions to customize the Plugin instance.
//
// Returns:
// - A pointer to the newly created Plugin instance.
//
// Example usage:
//
//	plugin := NewPlugin(
//		WithName("My Plugin"),
//		WithSlug("my-plugin"),
//		WithVersion("1.0.0"),
//		WithDescription("An example plugin."),
//	)
func NewPlugin(opts ...PluginOption) Plugin {
	p := defaultPlugin
	for _, opt := range opts {
		opt(&p)
	}
	return p
}
