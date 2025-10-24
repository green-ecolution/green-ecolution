package entities

import (
	"net/url"
)

type AuthPlugin struct {
	ClientID     string
	ClientSecret string
}

type Plugin struct {
	Slug        string `validate:"required"`
	Name        string `validate:"required"`
	Path        url.URL
	Version     string
	Description string
	Auth        AuthPlugin
}
