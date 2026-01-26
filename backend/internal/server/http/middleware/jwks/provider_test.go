package jwks

import (
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func generateTestKey(t *testing.T) *rsa.PrivateKey {
	t.Helper()
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err)
	return key
}

func base64EncodePublicKey(t *testing.T, key *rsa.PublicKey) string {
	t.Helper()
	pubKeyBytes, err := x509.MarshalPKIXPublicKey(key)
	require.NoError(t, err)
	return base64.StdEncoding.EncodeToString(pubKeyBytes)
}

func signJWT(t *testing.T, key *rsa.PrivateKey, kid string) string {
	t.Helper()
	token := jwt.New(jwt.SigningMethodRS256)
	if kid != "" {
		token.Header["kid"] = kid
	}
	tokenString, err := token.SignedString(key)
	require.NoError(t, err)
	return tokenString
}

// createJWKSServer creates a test HTTP server that serves a JWKS endpoint
func createJWKSServer(t *testing.T, key *rsa.PublicKey, kid string) *httptest.Server {
	t.Helper()

	// Create JWK from public key
	jwk := map[string]interface{}{
		"kty": "RSA",
		"use": "sig",
		"alg": "RS256",
		"kid": kid,
		"n":   base64.RawURLEncoding.EncodeToString(key.N.Bytes()),
		"e":   "AQAB", // 65537 in base64url
	}

	jwks := map[string]interface{}{
		"keys": []interface{}{jwk},
	}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(jwks)
	}))

	t.Cleanup(func() {
		server.Close()
	})

	return server
}

func TestNewProvider_WithStaticKeyOnly(t *testing.T) {
	// given
	key := generateTestKey(t)
	base64Key := base64EncodePublicKey(t, &key.PublicKey)

	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{
				StaticKey: base64Key,
			},
		},
	}

	// when
	provider, err := NewProvider(cfg)

	// then
	require.NoError(t, err)
	require.NotNil(t, provider)
	assert.NotNil(t, provider.staticKey)
	assert.Nil(t, provider.jwks)

	provider.Close()
}

func TestNewProvider_WithJWKSURL(t *testing.T) {
	// given
	key := generateTestKey(t)
	server := createJWKSServer(t, &key.PublicKey, "test-kid")

	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{
				JwksURL:         server.URL,
				RefreshInterval: 1 * time.Hour,
				RefreshTimeout:  5 * time.Second,
			},
		},
	}

	// when
	provider, err := NewProvider(cfg)

	// then
	require.NoError(t, err)
	require.NotNil(t, provider)
	assert.NotNil(t, provider.jwks)
	assert.Nil(t, provider.staticKey)

	provider.Close()
}

func TestNewProvider_NoKeySource(t *testing.T) {
	// given
	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{},
		},
	}

	// when
	provider, err := NewProvider(cfg)

	// then
	assert.Error(t, err)
	assert.Nil(t, provider)
	assert.ErrorIs(t, err, ErrNoKeySource)
}

func TestNewProvider_InvalidStaticKey(t *testing.T) {
	// given
	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{
				StaticKey: "invalid_base64_key",
			},
		},
	}

	// when
	provider, err := NewProvider(cfg)

	// then
	assert.Error(t, err)
	assert.Nil(t, provider)
}

func TestNewProvider_JWKSFetchFailedWithStaticFallback(t *testing.T) {
	// given
	key := generateTestKey(t)
	base64Key := base64EncodePublicKey(t, &key.PublicKey)

	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{
				StaticKey:      base64Key,
				JwksURL:        "http://invalid-url-that-does-not-exist.local/jwks",
				RefreshTimeout: 100 * time.Millisecond,
			},
		},
	}

	// when
	provider, err := NewProvider(cfg)

	// then
	require.NoError(t, err)
	require.NotNil(t, provider)
	assert.NotNil(t, provider.staticKey)
	assert.Nil(t, provider.jwks) // JWKS fetch failed, but we have static key fallback

	provider.Close()
}

func TestProvider_Keyfunc_WithJWKS(t *testing.T) {
	// given
	key := generateTestKey(t)
	kid := "test-kid-123"
	server := createJWKSServer(t, &key.PublicKey, kid)

	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{
				JwksURL:         server.URL,
				RefreshInterval: 1 * time.Hour,
				RefreshTimeout:  5 * time.Second,
			},
		},
	}

	provider, err := NewProvider(cfg)
	require.NoError(t, err)
	defer provider.Close()

	// Create and parse a signed JWT
	tokenString := signJWT(t, key, kid)
	token, err := jwt.Parse(tokenString, provider.Keyfunc)

	// then
	require.NoError(t, err)
	assert.True(t, token.Valid)
}

func TestProvider_Keyfunc_WithStaticKeyOnly(t *testing.T) {
	// given
	key := generateTestKey(t)
	base64Key := base64EncodePublicKey(t, &key.PublicKey)

	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{
				StaticKey: base64Key,
			},
		},
	}

	provider, err := NewProvider(cfg)
	require.NoError(t, err)
	defer provider.Close()

	// Create and parse a signed JWT (no kid in header)
	tokenString := signJWT(t, key, "")
	token, err := jwt.Parse(tokenString, provider.Keyfunc)

	// then
	require.NoError(t, err)
	assert.True(t, token.Valid)
}

func TestProvider_Keyfunc_FallbackToStaticKey(t *testing.T) {
	// given
	jwksKey := generateTestKey(t)
	staticKey := generateTestKey(t)
	base64Key := base64EncodePublicKey(t, &staticKey.PublicKey)

	// JWKS server has a different key (with kid "jwks-kid")
	server := createJWKSServer(t, &jwksKey.PublicKey, "jwks-kid")

	cfg := &config.IdentityAuthConfig{
		Enable: true,
		OidcProvider: config.OidcProvider{
			PublicKey: config.OidcPublicKey{
				StaticKey:       base64Key,
				JwksURL:         server.URL,
				RefreshInterval: 1 * time.Hour,
				RefreshTimeout:  5 * time.Second,
			},
		},
	}

	provider, err := NewProvider(cfg)
	require.NoError(t, err)
	defer provider.Close()

	// Sign token with static key (not in JWKS), without kid
	tokenString := signJWT(t, staticKey, "")
	token, err := jwt.Parse(tokenString, provider.Keyfunc)

	// Should fall back to static key
	require.NoError(t, err)
	assert.True(t, token.Valid)
}

type MockRoundTripper struct {
}

var _ http.RoundTripper = (*MockRoundTripper)(nil)

func (*MockRoundTripper) RoundTrip(req *http.Request) (*http.Response, error) {
	res := &http.Response{
		Proto:            "HTTP/1.1",
		ProtoMajor:       1,
		ProtoMinor:       1,
		Header:           http.Header{},
		TransferEncoding: []string{},
		Close:            false,
		Uncompressed:     false,
		Trailer:          http.Header{},
		Request:          req,
		TLS:              nil,
	}
	var data string
	var statusCode int
	if strings.HasPrefix(req.URL.Hostname(), "auth.example.com") {
		data = `{"jwks_uri":"https://auth.example.com/realms/test-realm/protocol/openid-connect/certs"}`
		statusCode = http.StatusOK
	} else {
		data = ""
		statusCode = http.StatusInternalServerError
	}

	res.Status = http.StatusText(statusCode)
	res.StatusCode = statusCode
	res.Body = io.NopCloser(strings.NewReader(data))
	res.ContentLength = int64(len(data))

	return res, nil
}

func TestResolveJWKSURL(t *testing.T) {
	client := http.Client{
		Transport: &MockRoundTripper{},
	}

	tests := []struct {
		name     string
		cfg      *config.OidcPublicKey
		baseURL  string
		realm    string
		expected string
	}{
		{
			name: "direct JWKS URL takes precedence",
			cfg: &config.OidcPublicKey{
				JwksURL: "https://direct.example.com/jwks",
			},
			baseURL:  "https://auth.example.com",
			realm:    "test-realm",
			expected: "https://direct.example.com/jwks",
		},
		{
			name:     "construct from base URL and realm",
			cfg:      &config.OidcPublicKey{},
			baseURL:  "https://auth.example.com",
			realm:    "test-realm",
			expected: "https://auth.example.com/realms/test-realm/protocol/openid-connect/certs",
		},
		{
			name:     "empty when no URL can be constructed",
			cfg:      &config.OidcPublicKey{},
			baseURL:  "",
			realm:    "",
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := &Provider{
				httpClient: &client,
				cfg:        tt.cfg,
				baseURL:    tt.baseURL,
				realm:      tt.realm,
			}

			result, _ := p.resolveJWKSURL()
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestParseStaticKey(t *testing.T) {
	t.Run("valid base64 key", func(t *testing.T) {
		key := generateTestKey(t)
		base64Key := base64EncodePublicKey(t, &key.PublicKey)

		parsed, err := parseStaticKey(base64Key)

		require.NoError(t, err)
		assert.NotNil(t, parsed)
	})

	t.Run("invalid base64", func(t *testing.T) {
		parsed, err := parseStaticKey("not-valid-base64!!!")

		assert.Error(t, err)
		assert.Nil(t, parsed)
		assert.ErrorIs(t, err, ErrStaticKeyParseFailed)
	})

	t.Run("valid base64 but invalid key format", func(t *testing.T) {
		invalidKey := base64.StdEncoding.EncodeToString([]byte("not a real key"))

		parsed, err := parseStaticKey(invalidKey)

		assert.Error(t, err)
		assert.Nil(t, parsed)
		assert.ErrorIs(t, err, ErrStaticKeyParseFailed)
	})
}

func TestGetInterval(t *testing.T) {
	assert.Equal(t, defaultRefreshInterval, getInterval(0))
	assert.Equal(t, 5*time.Minute, getInterval(5*time.Minute))
}

func TestGetTimeout(t *testing.T) {
	assert.Equal(t, defaultRefreshTimeout, getTimeout(0))
	assert.Equal(t, 30*time.Second, getTimeout(30*time.Second))
}
