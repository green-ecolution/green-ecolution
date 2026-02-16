package plugin_test

import (
	"net/url"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
)

var (
	TestPlugin = &entities.Plugin{
		Slug:        "test-plugin",
		Name:        "Test Plugin",
		Path:        url.URL{Scheme: "http", Host: "localhost:8080"},
		Version:     "1.0.0",
		Description: "A test plugin",
		Auth: entities.AuthPlugin{
			ClientID:     "test-client-id",
			ClientSecret: "test-client-secret",
		},
	}

	TestPlugin2 = &entities.Plugin{
		Slug:        "test-plugin-2",
		Name:        "Test Plugin 2",
		Path:        url.URL{Scheme: "http", Host: "localhost:9090"},
		Version:     "2.0.0",
		Description: "Another test plugin",
		Auth: entities.AuthPlugin{
			ClientID:     "test-client-id-2",
			ClientSecret: "test-client-secret-2",
		},
	}

	TestAuthPlugin = &entities.AuthPlugin{
		ClientID:     "test-client-id",
		ClientSecret: "test-client-secret",
	}

	TestClientToken = &entities.ClientToken{
		AccessToken:  "test-access-token",
		ExpiresIn:    3600,
		RefreshToken: "test-refresh-token",
		Expiry:       time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
		TokenType:    "Bearer",
	}
)
