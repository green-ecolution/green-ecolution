package auth

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/url"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/spf13/viper"
)

// AuthDummyService is used to disable the auth service by configuration
type AuthDummyService struct {
	repo storage.UserRepository
}

func NewDummyAuthService(repo storage.UserRepository) service.AuthService {
	return &AuthDummyService{
		repo: repo,
	}
}

func (s *AuthDummyService) Ready() bool {
	return true
}

func (s *AuthDummyService) LoginRequest(ctx context.Context, loginRequest *entities.LoginRequest) *entities.LoginResp {
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

	return &entities.LoginResp{
		LoginURL: dummyURL,
	}
}

func (s *AuthDummyService) LogoutRequest(_ context.Context, _ *entities.Logout) error {
	return nil
}

func (s *AuthDummyService) ClientTokenCallback(_ context.Context, _ *entities.LoginCallback) (*entities.ClientToken, error) {
	return s.generateDummyToken()
}

func (s *AuthDummyService) Register(_ context.Context, _ *entities.RegisterUser) (*entities.User, error) {
	return nil, service.NewError(service.Gone, "auth service is disabled")
}

func (s *AuthDummyService) RetrospectToken(_ context.Context, _ string) (*entities.IntroSpectTokenResult, error) {
	return nil, service.NewError(service.Gone, "auth service is disabled")
}

func (s *AuthDummyService) RefreshToken(_ context.Context, _ string) (*entities.ClientToken, error) {
	return s.generateDummyToken()
}

func (s *AuthDummyService) GetAll(ctx context.Context) ([]*entities.User, error) {
	return s.repo.GetAll(ctx)
}

func (s *AuthDummyService) GetByIDs(ctx context.Context, ids []string) ([]*entities.User, error) {
	return s.repo.GetByIDs(ctx, ids)
}

func (s *AuthDummyService) GetAllByRole(ctx context.Context, role entities.UserRole) ([]*entities.User, error) {
	return s.repo.GetAllByRole(ctx, role)
}

func (s *AuthDummyService) generateDummyToken() (*entities.ClientToken, error) {
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

	return &entities.ClientToken{
		TokenType:    "Bearer",
		Expiry:       time.Now().Add(365 * 24 * time.Hour),
		ExpiresIn:    int(365 * 24 * time.Hour / time.Second),
		AccessToken:  fmt.Sprintf("lsidu.%s.oicsxfusd", b64Buf),
		RefreshToken: fmt.Sprintf("sinxoled.%s.sldkfjalf", b64Buf),
	}, nil
}
