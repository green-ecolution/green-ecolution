package middleware

import (
	"bytes"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"log"
	"net/http/httptest"
	"testing"

	"github.com/gofiber/fiber/v2"
	golangJwt "github.com/golang-jwt/jwt/v5"
	"github.com/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/backend/internal/entities"
	serviceMock "github.com/green-ecolution/backend/internal/service/_mock"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
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

func base64EncodePublicKey(key *rsa.PublicKey) string {
	pubKeyBytes, err := x509.MarshalPKIXPublicKey(key)
	if err != nil {
		log.Fatalf("Failed to marshal public key: %v", err)
	}
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

func Test_NewJWTMiddleware(t *testing.T) {
	t.Run("should return a new JWT middleware", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(&validKey.PublicKey)
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

	t.Run("should return a handler with error on invalid public key", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		cfg := &config.IdentityAuthConfig{
			Enable: true,
			OidcProvider: config.OidcProvider{
				PublicKey: config.OidcPublicKey{
					StaticKey: "invalid_base64_encoded_key",
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
		assert.Contains(t, body.String(), "failed to parse public key")
	})
}

func Test_parsePublicKey(t *testing.T) {
	t.Run("should return a public key", func(t *testing.T) {
		// given
		base64Key := base64EncodePublicKey(&validKey(t).PublicKey)

		// when
		got, err := parsePublicKey(base64Key)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, got)
	})

	t.Run("should return error on invalid base64 key", func(t *testing.T) {
		// given
		invalidBase64Key := "invalid_base64_encoded_key"

		// when
		got, err := parsePublicKey(invalidBase64Key)

		// then
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func Test_successHandler(t *testing.T) {
	t.Run("should success on introspect token", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(&validKey.PublicKey)
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

	t.Run("should return code 500 on error", func(t *testing.T) {
		// given
		mockSvc := serviceMock.NewMockAuthService(t)
		app := fiber.New()
		handler := NewJWTMiddleware(&config.IdentityAuthConfig{
			Enable: true,
		}, mockSvc)
		app.Use(handler)
		app.Get("/", func(c *fiber.Ctx) error {
			return c.SendString("Hello, World!")
		})

		// when
		req := httptest.NewRequest(fiber.MethodGet, "/", nil)
		resp, _ := app.Test(req)
		defer resp.Body.Close()

		// then
		assert.Equal(t, fiber.StatusInternalServerError, resp.StatusCode)
	})

	t.Run("should return code 401 on inactive token", func(t *testing.T) {
		// given
		authSvc := serviceMock.NewMockAuthService(t)
		validKey := validKey(t)
		base64Key := base64EncodePublicKey(&validKey.PublicKey)
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
}
