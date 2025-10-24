package plugin

import "time"

type PluginRegisterRequest struct {
	Slug        string     `json:"slug"`
	Name        string     `json:"name"`
	Description string     `json:"description"`
	Version     string     `json:"version"`
	Path        string     `json:"path"`
	Auth        PluginAuth `json:"auth"`
}

type PluginAuth struct {
	ClientID     string `json:"client_id"`
	ClientSecret string `json:"client_secret"`
}

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
}
