package keycloak

import (
	"context"
	"time"

	"github.com/Nerzal/gocloak/v13"
	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
)

func (r *KeycloakRepository) RetrospectToken(ctx context.Context, token string) (*entities.IntroSpectTokenResult, error) {
	log := logger.GetLogger(ctx)
	client := gocloak.NewClient(r.cfg.OidcProvider.BaseURL)

	rptResult, err := client.RetrospectToken(ctx, token, r.cfg.OidcProvider.Frontend.ClientID, r.cfg.OidcProvider.Frontend.ClientSecret, r.cfg.OidcProvider.DomainName)
	if err != nil {
		log.Error("failed to retrospect given token", "error", err, "token", token)
		return nil, err
	}

	return &entities.IntroSpectTokenResult{
		Active:   rptResult.Active,
		Exp:      rptResult.Exp,
		AuthTime: rptResult.AuthTime,
		Type:     rptResult.Type,
	}, nil
}

func (r *KeycloakRepository) GetAccessTokenFromClientCode(ctx context.Context, code, redirectURL string) (*entities.ClientToken, error) {
	log := logger.GetLogger(ctx)
	client := gocloak.NewClient(r.cfg.OidcProvider.BaseURL)

	tokenOptions := gocloak.TokenOptions{
		ClientID:     &r.cfg.OidcProvider.Frontend.ClientID,
		ClientSecret: &r.cfg.OidcProvider.Frontend.ClientSecret,
		Code:         &code,
		GrantType:    gocloak.StringP("authorization_code"),
		RedirectURI:  &redirectURL,
	}

	token, err := client.GetToken(ctx, r.cfg.OidcProvider.DomainName, tokenOptions)
	if err != nil {
		log.Error("failed to receive access token from client code in keycloak", "error", err, "redirect_url", redirectURL, "access_code", code)
		return nil, err
	}

	return &entities.ClientToken{
		AccessToken:      token.AccessToken,
		RefreshToken:     token.RefreshToken,
		ExpiresIn:        token.ExpiresIn,
		Expiry:           time.Now().Add(time.Duration(token.ExpiresIn) * time.Second).Add(-1 * time.Second),
		RefreshExpiresIn: token.RefreshExpiresIn,
		TokenType:        token.TokenType,
		NotBeforePolicy:  token.NotBeforePolicy,
		SessionState:     token.SessionState,
		Scope:            token.Scope,
	}, nil
}

func (r *KeycloakRepository) RefreshToken(ctx context.Context, refreshToken string) (*entities.ClientToken, error) {
	log := logger.GetLogger(ctx)
	client := gocloak.NewClient(r.cfg.OidcProvider.BaseURL)
	token, err := client.RefreshToken(ctx, refreshToken, r.cfg.OidcProvider.Frontend.ClientID, r.cfg.OidcProvider.Frontend.ClientSecret, r.cfg.OidcProvider.DomainName)
	if err != nil {
		log.Error("failed to refresh given token", "error", err, "refresh_token", refreshToken)
		return nil, err
	}

	log.Debug("refreshed token successfully")

	return &entities.ClientToken{
		AccessToken:      token.AccessToken,
		RefreshToken:     token.RefreshToken,
		ExpiresIn:        token.ExpiresIn,
		RefreshExpiresIn: token.RefreshExpiresIn,
		TokenType:        token.TokenType,
		NotBeforePolicy:  token.NotBeforePolicy,
		SessionState:     token.SessionState,
		Scope:            token.Scope,
		Expiry:           time.Now().Add(time.Duration(token.ExpiresIn) * time.Second).Add(-1 * time.Second),
	}, nil
}

func (r *KeycloakRepository) GetAccessTokenFromClientCredentials(ctx context.Context, clientID, clientSecret string) (*entities.ClientToken, error) {
	log := logger.GetLogger(ctx)
	client := gocloak.NewClient(r.cfg.OidcProvider.BaseURL)

	tokenOptions := gocloak.TokenOptions{
		ClientID:     &clientID,
		ClientSecret: &clientSecret,
		GrantType:    gocloak.StringP("client_credentials"),
	}

	token, err := client.GetToken(ctx, r.cfg.OidcProvider.DomainName, tokenOptions)
	if err != nil {
		log.Error("failed to get token from client credentials", "error", err, "client_id", clientID, "client_secret", "*********")
		return nil, err
	}

	return &entities.ClientToken{
		AccessToken:      token.AccessToken,
		RefreshToken:     token.RefreshToken,
		ExpiresIn:        token.ExpiresIn,
		RefreshExpiresIn: token.RefreshExpiresIn,
		TokenType:        token.TokenType,
		NotBeforePolicy:  token.NotBeforePolicy,
		SessionState:     token.SessionState,
		Expiry:           time.Now().Add(time.Duration(token.ExpiresIn) * time.Second).Add(-1 * time.Second),
		Scope:            token.Scope,
	}, nil
}
