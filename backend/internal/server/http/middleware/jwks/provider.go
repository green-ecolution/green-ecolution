package jwks

import (
	"context"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"sync"
	"time"

	"github.com/MicahParks/keyfunc/v2"
	"github.com/golang-jwt/jwt/v5"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
)

const (
	refreshRateLimit = 5 * time.Second
)

var (
	ErrNoKeySource          = errors.New("no key source configured (neither JWKS nor static key)")
	ErrJWKSFetchFailed      = errors.New("failed to fetch JWKS")
	ErrStaticKeyParseFailed = errors.New("failed to parse static public key")
)

// Provider manages JWKS fetching with caching and fallback to static key
type Provider struct {
	cfg     *config.OidcPublicKey
	baseURL string
	realm   string

	mu        sync.RWMutex
	jwks      *keyfunc.JWKS
	staticKey *rsa.PublicKey

	httpClient *http.Client
}

// NewProvider creates a JWKS provider with the given configuration.
// It will attempt to initialize JWKS fetching and fall back to static key if available.
func NewProvider(cfg *config.IdentityAuthConfig) (*Provider, error) {
	p := &Provider{
		cfg:     &cfg.OidcProvider.PublicKey,
		baseURL: cfg.OidcProvider.BaseURL,
		realm:   cfg.OidcProvider.DomainName,
		httpClient: &http.Client{
			Timeout: cfg.OidcProvider.PublicKey.RefreshTimeout,
		},
	}

	if cfg.OidcProvider.PublicKey.StaticKey != "" {
		key, err := parseStaticKey(cfg.OidcProvider.PublicKey.StaticKey)
		if err != nil {
			slog.Error("static key parsing failed, will rely on JWKS", "error", err)
			return nil, err
		}
		p.staticKey = key
		slog.Debug("static key initialized as fallback")
	} else {
		if err := p.initJWKS(); err != nil {
			slog.Error("JWKS initialization failed, using static key fallback", "error", err)
			return nil, err
		}
	}

	if p.jwks == nil && p.staticKey == nil {
		return nil, ErrNoKeySource
	}

	return p, nil
}

// Keyfunc returns a jwt.Keyfunc for use with JWT validation middleware
func (p *Provider) Keyfunc(token *jwt.Token) (any, error) {
	p.mu.RLock()
	defer p.mu.RUnlock()

	// Try JWKS first
	if p.jwks != nil {
		key, err := p.jwks.Keyfunc(token)
		if err == nil {
			return key, nil
		}

		// If key not found in JWKS and static fallback exists, use it
		if p.staticKey != nil {
			slog.Debug("key not found in JWKS, using static key fallback", "error", err)
			return p.staticKey, nil
		}

		return nil, err
	}

	// Fallback to static key only
	if p.staticKey != nil {
		return p.staticKey, nil
	}

	return nil, ErrNoKeySource
}

// Close shuts down background refresh goroutines
func (p *Provider) Close() {
	p.mu.Lock()
	defer p.mu.Unlock()

	if p.jwks != nil {
		p.jwks.EndBackground()
		slog.Info("JWKS provider shut down")
	}
}

func (p *Provider) initJWKS() error {
	jwksURL, err := p.resolveJWKSURL()
	if err != nil {
		slog.Error("no JWKS URL resolved", "error", err)
		return err
	}

	opts := keyfunc.Options{
		Client:                      p.httpClient,
		RefreshInterval:             p.cfg.RefreshInterval,
		RefreshTimeout:              p.cfg.RefreshTimeout,
		RefreshRateLimit:            refreshRateLimit,
		TolerateInitialJWKHTTPError: true,
		RefreshUnknownKID:           true,
		RefreshErrorHandler: func(err error) {
			slog.Error("JWKS refresh failed", "error", err, "url", jwksURL)
		},
		ResponseExtractor: func(ctx context.Context, res *http.Response) (json.RawMessage, error) {
			var err error
			defer res.Body.Close()

			var openIDCfg struct {
				JWKsURI string `json:"jwks_uri"`
			}
			if err = json.NewDecoder(res.Body).Decode(&openIDCfg); err != nil {
				return nil, fmt.Errorf("error decoding openid config: %w", err)
			}

			if openIDCfg.JWKsURI == "" {
				return nil, fmt.Errorf("jwksURI of openid config is empty")
			}

			req, err := http.NewRequestWithContext(ctx, http.MethodGet, openIDCfg.JWKsURI, http.NoBody)
			if err != nil {
				return nil, fmt.Errorf("jwksURI of openid config is invalid: %w", err)
			}

			res2, err := p.httpClient.Do(req)
			if err != nil {
				return nil, err
			}
			defer res2.Body.Close()

			return keyfunc.ResponseExtractorStatusOK(ctx, res2)
		},
	}

	slog.Info("initializing JWKS provider", "url", jwksURL, "refresh_interval", opts.RefreshInterval)

	jwks, err := keyfunc.Get(jwksURL, opts)
	if err != nil {
		return fmt.Errorf("%w: %v", ErrJWKSFetchFailed, err)
	}

	p.mu.Lock()
	p.jwks = jwks
	p.mu.Unlock()

	slog.Info("JWKS provider initialized successfully", "keys", jwks.Len())
	return nil
}

func (p *Provider) resolveJWKSURL() (string, error) {
	if p.cfg.JwksURL != "" {
		return p.cfg.JwksURL, nil
	}
	if p.baseURL == "" || p.realm == "" {
		return "", ErrNoKeySource
	}

	oidcBaseURL := fmt.Sprintf("%s/realms/%s", p.baseURL, p.realm)
	return fmt.Sprintf("%s/.well-known/openid-configuration", oidcBaseURL), nil
}

func parseStaticKey(base64Str string) (*rsa.PublicKey, error) {
	buf, err := base64.StdEncoding.DecodeString(base64Str)
	if err != nil {
		return nil, fmt.Errorf("%w: base64 decode: %v", ErrStaticKeyParseFailed, err)
	}

	parsedKey, err := x509.ParsePKIXPublicKey(buf)
	if err != nil {
		return nil, fmt.Errorf("%w: PKIX parse: %v", ErrStaticKeyParseFailed, err)
	}

	publicKey, ok := parsedKey.(*rsa.PublicKey)
	if !ok {
		return nil, fmt.Errorf("%w: key is not RSA public key", ErrStaticKeyParseFailed)
	}

	return publicKey, nil
}
