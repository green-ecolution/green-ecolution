package entities

import "time"

type LoginResponse struct {
	LoginURL string `json:"login_url"`
} //	@Name	LoginResponse

type LoginTokenRequest struct {
	Code string `json:"code" validate:"required"`
} //	@Name	LoginTokenRequest

type LogoutRequest struct {
	RefreshToken string `json:"refresh_token" validate:"required"`
} //	@Name	LogoutRequest

type ClientTokenResponse struct {
	AccessToken      string    `json:"access_token"`
	IDToken          string    `json:"id_token"`
	ExpiresIn        int       `json:"expires_in"`
	Expiry           time.Time `json:"expiry"`
	RefreshExpiresIn int       `json:"refresh_expires_in"`
	RefreshToken     string    `json:"refresh_token"`
	TokenType        string    `json:"token_type"`
	NotBeforePolicy  int       `json:"not_before_policy"`
	SessionState     string    `json:"session_state"`
	Scope            string    `json:"scope"`
} //	@Name	ClientToken

type RefreshTokenRequest struct {
	RefreshToken string `json:"refresh_token" validate:"required"`
} //	@Name	RefreshTokenRequest
