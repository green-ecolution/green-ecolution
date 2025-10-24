package plugin

import (
	"context"
	"errors"
	"net/http"
	"net/url"
	"time"
)

// PluginWorkerConfig defines the configuration settings for a PluginWorker.
//
// Fields:
// - plugin: The plugin being managed by the worker.
// - host: The URL of the plugin host where the plugin communicates.
// - hostAPIVersion: The API version used by the plugin host.
// - interval: The interval for periodic tasks, such as heartbeats.
// - client: The HTTP client used for making requests to the plugin host.
// - token: The authentication token used for secure communication.
// - clientID: The client id from the oidc provider
// - clientSecret: The client secret from the oidc provider
type PluginWorkerConfig struct {
	plugin         Plugin
	host           *url.URL
	hostAPIVersion string
	interval       time.Duration
	client         *http.Client
	token          *Token
	clientID       string
	clientSecret   string
}

// PluginWorker is responsible for managing the lifecycle of a plugin.
//
// This includes tasks such as:
// - Registering the plugin with the host.
// - Sending periodic heartbeats to indicate the plugin is active.
// - Maintaining and using an authentication token for secure communication.
type PluginWorker struct {
	cfg PluginWorkerConfig
}

// PluginWorkerOption is a functional option for configuring a PluginWorker.
//
// Functional options provide flexibility by allowing configuration settings to be passed dynamically at runtime.
type PluginWorkerOption func(*PluginWorkerConfig)

// WithClient sets the HTTP client for the PluginWorker.
//
// Example usage:
//
//	worker, err := NewPluginWorker(
//		WithClient(http.DefaultClient),
//	)
func WithClient(client *http.Client) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.client = client
	}
}

// WithHost sets the host URL for the PluginWorker.
//
// Example usage:
//
//	hostURL, _ := url.Parse("https://example.com")
//	worker, err := NewPluginWorker(
//		WithHost(hostURL),
//	)
func WithHost(host *url.URL) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.host = host
	}
}

// WithHostAPIVersion sets the API version for the PluginWorker.
//
// Example usage:
//
//	worker, err := NewPluginWorker(
//		WithHostAPIVersion("v2"),
//	)
func WithHostAPIVersion(version string) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.hostAPIVersion = version
	}
}

// WithPlugin sets the plugin for the PluginWorker.
//
// Example usage:
//
//	plugin := Plugin{Name: "Example Plugin"}
//	worker, err := NewPluginWorker(
//		WithPlugin(plugin),
//	)
func WithPlugin(plugin Plugin) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.plugin = plugin
	}
}

// WithInterval sets the interval for periodic tasks, such as heartbeats.
//
// Example usage:
//
//	worker, err := NewPluginWorker(
//		WithInterval(5 * time.Minute),
//	)
func WithInterval(interval time.Duration) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.interval = interval
	}
}

// WithToken sets the authentication token for the PluginWorker.
//
// Example usage:
//
//	token := &Token{AccessToken: "abc123"}
//	worker, err := NewPluginWorker(
//		WithToken(token),
//	)
func WithToken(token *Token) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.token = token
	}
}

// WithClientID sets the client id from oidc provider for the PluginWorker.
func WithClientID(clientID string) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.clientID = clientID
	}
}

// WithClientSecret sets the client secret from oidc provider for the PluginWorker.
func WithClientSecret(clientSecret string) PluginWorkerOption {
	return func(cfg *PluginWorkerConfig) {
		cfg.clientSecret = clientSecret
	}
}

// IsValid checks whether the PluginWorkerConfig is valid.
//
// Returns true if all required fields are properly set (e.g., host, plugin, interval, client), false otherwise.
func (c *PluginWorkerConfig) IsValid() bool {
	return c.host != nil && c.plugin.PluginHostPath != nil && c.interval > 0 && c.client != nil && c.plugin.Name != ""
}

// defaultCfg provides default values for PluginWorker configurations.
var defaultCfg = PluginWorkerConfig{
	client:         http.DefaultClient,
	interval:       2 * time.Minute,
	hostAPIVersion: "v1",
}

// NewPluginWorker creates a new PluginWorker instance with the provided options.
//
// If no options are provided, the worker is created with default values.
// This function validates the configuration and returns an error if the configuration is invalid.
//
// Parameters:
// - opts: A variadic list of PluginWorkerOption functions to customize the PluginWorker.
//
// Returns:
// - A pointer to the PluginWorker instance if successful.
// - An error if the configuration is invalid.
//
// Example usage:
//
//	hostURL, _ := url.Parse("https://example.com")
//	worker, err := NewPluginWorker(
//		WithHost(hostURL),
//		WithInterval(1*time.Minute),
//	)
//	if err != nil {
//		log.Fatalf("Failed to create PluginWorker: %v", err)
//	}
func NewPluginWorker(opts ...PluginWorkerOption) (*PluginWorker, error) {
	cfg := defaultCfg
	for _, opt := range opts {
		opt(&cfg)
	}
	if !cfg.IsValid() {
		return nil, errors.New("invalid config")
	}

	return &PluginWorker{cfg: cfg}, nil
}

// RunHeartbeat starts the heartbeat runner for the PluginWorker.
//
// This function sends periodic heartbeat signals to the plugin host at the configured interval.
// It uses a ticker to manage timing and stops when the provided context is canceled.
//
// Parameters:
// - ctx: The context for managing lifecycle and cancellation of the heartbeat runner.
//
// Returns:
// - nil when the context is canceled.
// - An error if sending a heartbeat fails.
//
// Example usage:
//
//	ctx, cancel := context.WithCancel(context.Background())
//	defer cancel()
//
//	err := worker.RunHeartbeat(ctx)
//	if err != nil {
//		log.Fatalf("Heartbeat failed: %v", err)
//	}
func (w *PluginWorker) RunHeartbeat(ctx context.Context) error {
	ticker := time.NewTicker(w.cfg.interval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			if err := w.Heartbeat(ctx); err != nil {
				return err
			}
		}
	}
}
