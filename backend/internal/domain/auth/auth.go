package auth

import (
	"context"
	"net/url"
	"time"
)

type IntroSpectTokenResult struct {
	Exp      *int
	Active   *bool
	AuthTime *int
	Type     *string
}

type ClientToken struct {
	AccessToken      string
	IDToken          string
	Expiry           time.Time
	ExpiresIn        int
	RefreshExpiresIn int
	RefreshToken     string
	TokenType        string
	NotBeforePolicy  int
	SessionState     string
	Scope            string
}

type LoginRequest struct {
	RedirectURL *url.URL
}

type LoginResp struct {
	LoginURL *url.URL
}

type LoginCallback struct {
	Code        string
	RedirectURL *url.URL
}

type Logout struct {
	RefreshToken string
}

type AuthRepository interface {
	RetrospectToken(ctx context.Context, token string) (*IntroSpectTokenResult, error)
	GetAccessTokenFromClientCode(ctx context.Context, code, redirectURL string) (*ClientToken, error)
	RefreshToken(ctx context.Context, refreshToken string) (*ClientToken, error)
	GetAccessTokenFromClientCredentials(ctx context.Context, clientID, clientSecret string) (*ClientToken, error)
}
