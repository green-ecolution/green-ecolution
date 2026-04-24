package auth

import "context"

type AuthRepository interface {
	RetrospectToken(ctx context.Context, token string) (*IntroSpectTokenResult, error)
	GetAccessTokenFromClientCode(ctx context.Context, code, redirectURL string) (*ClientToken, error)
	RefreshToken(ctx context.Context, refreshToken string) (*ClientToken, error)
	GetAccessTokenFromClientCredentials(ctx context.Context, clientID, clientSecret string) (*ClientToken, error)
}
