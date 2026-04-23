package auth

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/url"
	"time"

	"github.com/spf13/viper"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

// AuthDummyService is used to disable the auth service by configuration
type AuthDummyService struct {
	repo shared.UserRepository
}

func NewDummyAuthService(repo shared.UserRepository) ports.AuthService {
	return &AuthDummyService{
		repo: repo,
	}
}

func (s *AuthDummyService) Ready() bool {
	return true
}

func (s *AuthDummyService) LoginRequest(ctx context.Context, loginRequest *shared.LoginRequest) *shared.LoginResp {
	log := logger.GetLogger(ctx)
	appURLRaw := viper.GetString("server.app_url")
	dummyURL, err := url.Parse(appURLRaw + "/api/v1/user/auth/dummy")
	if err != nil {
		log.Error("failed to parse app url in config", "error", err, "app_url", appURLRaw)
		panic("failed to parse app url in config. Pleas check your configuration")
	}
	query := dummyURL.Query()
	query.Add("redirect_uri", loginRequest.RedirectURL.String())
	dummyURL.RawQuery = query.Encode()

	return &shared.LoginResp{
		LoginURL: dummyURL,
	}
}

func (s *AuthDummyService) LogoutRequest(_ context.Context, _ *shared.Logout) error {
	return nil
}

func (s *AuthDummyService) ClientTokenCallback(_ context.Context, _ *shared.LoginCallback) (*shared.ClientToken, error) {
	return s.generateDummyToken()
}

func (s *AuthDummyService) Register(_ context.Context, _ *shared.RegisterUser) (*shared.User, error) {
	return nil, ports.NewError(ports.Gone, "auth service is disabled")
}

func (s *AuthDummyService) RetrospectToken(_ context.Context, _ string) (*shared.IntroSpectTokenResult, error) {
	return nil, ports.NewError(ports.Gone, "auth service is disabled")
}

func (s *AuthDummyService) RefreshToken(_ context.Context, _ string) (*shared.ClientToken, error) {
	return s.generateDummyToken()
}

func (s *AuthDummyService) GetAll(ctx context.Context) ([]*shared.User, error) {
	return s.repo.GetAll(ctx)
}

func (s *AuthDummyService) GetByIDs(ctx context.Context, ids []string) ([]*shared.User, error) {
	return s.repo.GetByIDs(ctx, ids)
}

func (s *AuthDummyService) GetAllByRole(ctx context.Context, role shared.UserRole) ([]*shared.User, error) {
	return s.repo.GetAllByRole(ctx, role)
}

func (s *AuthDummyService) generateDummyToken() (*shared.ClientToken, error) {
	var buf bytes.Buffer
	err := json.NewEncoder(&buf).Encode(map[string]interface{}{
		"email":              "toni.tester@green-ecolution.de",
		"preferred_username": "ttester",
		"given_name":         "Toni",
		"family_name":        "Tester",
		"driving_licenses":   []string{"B", "BE", "C", "CE"},
		"user_roles":         []string{"green-ecolution"},
		"status":             "available",
	})

	if err != nil {
		return nil, err
	}

	b64Buf := base64.RawURLEncoding.EncodeToString(buf.Bytes())

	return &shared.ClientToken{
		TokenType:    "Bearer",
		Expiry:       time.Now().Add(365 * 24 * time.Hour),
		ExpiresIn:    int(365 * 24 * time.Hour / time.Second),
		AccessToken:  fmt.Sprintf("lsidu.%s.oicsxfusd", b64Buf),
		RefreshToken: fmt.Sprintf("sinxoled.%s.sldkfjalf", b64Buf),
	}, nil
}
