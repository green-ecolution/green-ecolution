package middleware

import (
	"bytes"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gofiber/fiber/v2"
	golangJwt "github.com/golang-jwt/jwt/v5"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

func validKey(t testing.TB) *rsa.PrivateKey {
	t.Helper()
	t.Log("Generating a valid public key")
	random := rand.Reader
	key, err := rsa.GenerateKey(random, 2048)
	if err != nil {
		t.Fatalf("Failed to generate key: %v", err)
	}

	return key
}

func base64EncodePublicKey(t testing.TB, key *rsa.PublicKey) string {
	t.Helper()
	pubKeyBytes, err := x509.MarshalPKIXPublicKey(key)
	require.NoError(t, err)
	return base64.StdEncoding.EncodeToString(pubKeyBytes)
}

func signJWT(t *testing.T, key *rsa.PrivateKey) string {
	t.Helper()
	token := golangJwt.New(golangJwt.SigningMethodRS256)
	tokenString, err := token.SignedString(key)
	if err != nil {
		t.Fatalf("Failed to sign token: %v", err)
	}

	return tokenString
}

func signJWTWithKid(t *testing.T, key *rsa.PrivateKey, kid string) string {
	t.Helper()
	token := golangJwt.New(golangJwt.SigningMethodRS256)
	token.Header["kid"] = kid
	tokenString, err := token.SignedString(key)
	require.NoError(t, err)
	return tokenString
}

func createJWKSServer(t *testing.T, key *rsa.PublicKey, kid string) *httptest.Server {
	t.Helper()
	jwk := map[string]interface{}{
		"kty": "RSA",
		"use": "sig",
		"alg": "RS256",
		"kid": kid,
		"n":   base64.RawURLEncoding.EncodeToString(key.N.Bytes()),
		"e":   "AQAB",
	}
	jwks := map[string]interface{}{"keys": []interface{}{jwk}}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(jwks)
	}))
	t.Cleanup(server.Close)
	return server
}

func Test_NewJWTMiddleware(t *testing.T) {
	t.Run("should return a new JWT middleware", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(t, &validKey.PublicKey)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: base64Key,
				},
			},
		}

		// when
		got := NewJWTMiddleware(cfg, authSvc)

		// then
		assert.NotNil(t, got)
	})

	t.Run("should return a handler with error on no key source configured", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: "", // empty - no static key
					JwksURL:   "", // empty - no JWKS URL
				},
			},
		}

		// when
		middleware := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(middleware)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		body := new(bytes.Buffer)
		_, err := body.ReadFrom(resp.Body)
		if err != nil {
			t.Fatalf("Failed to read response body: %v", err)
		}

		// then
		assert.Equal(t, fiber.StatusInternalServerError, resp.StatusCode)
		assert.Contains(t, body.String(), "failed to initialize authentication")
	})
}

func Test_JWTMiddleware_TokenValidation(t *testing.T) {
	t.Run("should success on introspect token", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(t, &validKey.PublicKey)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: base64Key,
				},
			},
		}

		// when
		authSvc.EXPECT().RetrospectToken(mock.Anything, mock.Anything).Return(&entities.IntroSpectTokenResult{Active: utils.P(true)}, nil)
		got := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(got)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		req.Header.Set("Authorization", "Bearer "+signJWT(t, validKey))
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then
		assert.NotNil(t, got)
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)
	})

	t.Run("should return code 401 on inactive token", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(t, &validKey.PublicKey)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: base64Key,
				},
			},
		}

		// when
		authSvc.EXPECT().RetrospectToken(mock.Anything, mock.Anything).Return(&entities.IntroSpectTokenResult{Active: utils.P(false)}, nil)
		got := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(got)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		req.Header.Set("Authorization", "Bearer "+signJWT(t, validKey))
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then
		assert.NotNil(t, got)
		assert.Equal(t, fiber.StatusUnauthorized, resp.StatusCode)
	})

	t.Run("should pass through when auth is disabled", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		cfg := &config.IdentityAuthConfig{
			Enable: false,
		}

		// when
		middleware := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(middleware)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		// No Authorization header
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)
	})

	t.Run("should return 401 on invalid JWT signature", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		correctKey := validKey(t)
		wrongKey := validKey(t) // Different key for signing
		base64Key := base64EncodePublicKey(t, &correctKey.PublicKey)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: base64Key,
				},
			},
		}

		// when
		middleware := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(middleware)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		req.Header.Set("Authorization", "Bearer "+signJWT(t, wrongKey))
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then
		assert.Equal(t, fiber.StatusUnauthorized, resp.StatusCode)
	})

	t.Run("should return 401 on missing authorization header", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(t, &validKey.PublicKey)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: base64Key,
				},
			},
		}

		// when
		middleware := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(middleware)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		// No Authorization header set
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then
		assert.Equal(t, fiber.StatusUnauthorized, resp.StatusCode)
	})

	t.Run("should return 401 on malformed authorization header", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(t, &validKey.PublicKey)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: base64Key,
				},
			},
		}

		middleware := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(middleware)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		testCases := []struct {
			name   string
			header string
		}{
			{"empty bearer", "Bearer "},
			{"invalid scheme", "Basic dXNlcjpwYXNz"},
			{"no scheme", "some-random-token"},
		}

		for _, tc := range testCases {
			t.Run(tc.name, func(t *testing.T) {
				req := httptest.NewRequest(fiber.MethodGet, "/", nil)
				req.Header.Set("Authorization", tc.header)
				resp, _ := app.Test(req)
				defer resp.Body.Close()

				assert.Equal(t, fiber.StatusUnauthorized, resp.StatusCode)
			})
		}
	})

	t.Run("should return error when RetrospectToken fails", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(t, &validKey.PublicKey)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: base64Key,
				},
			},
		}

		// when
		authSvc.EXPECT().RetrospectToken(mock.Anything, mock.Anything).Return(nil, errors.New("keycloak unavailable"))
		middleware := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(middleware)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		req.Header.Set("Authorization", "Bearer "+signJWT(t, validKey))
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then - should return error status (not 200)
		assert.NotEqual(t, fiber.StatusOK, resp.StatusCode)
	})

	t.Run("should work with JWKS URL instead of static key", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		testKey := validKey(t)
		kid := "test-key-id"
		jwksServer := createJWKSServer(t, &testKey.PublicKey, kid)

		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					JwksURL:         jwksServer.URL,
					RefreshInterval: 1 * time.Hour,
					RefreshTimeout:  5 * time.Second,
				},
			},
		}

		// when
		authSvc.EXPECT().RetrospectToken(mock.Anything, mock.Anything).Return(&entities.IntroSpectTokenResult{Active: utils.P(true)}, nil)
		middleware := NewJWTMiddleware(cfg, authSvc)
		app := fiber.New()
		app.Use(middleware)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		req.Header.Set("Authorization", "Bearer "+signJWTWithKid(t, testKey, kid))
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then
		assert.Equal(t, fiber.StatusOK, resp.StatusCode)
	})
}

func Test_CloseJWKSProvider(t *testing.T) {
	t.Run("should not panic when provider is nil", func(t *testing.T) {
		// given
		jwksProvider = nil

		// when/then - should not panic
		assert.NotPanics(t, func() {
			CloseJWKSProvider()
		})
	})
}
