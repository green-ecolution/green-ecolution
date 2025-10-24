package entities

import (
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
	Code        string `validate:"required"`
	RedirectURL *url.URL
}

type Logout struct {
	RefreshToken string `validate:"required"`
}
