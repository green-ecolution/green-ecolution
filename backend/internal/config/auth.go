package config

import "time"

type OidcProvider struct {
	BaseURL    string        `mapstructure:"base_url"`
	HealthURL  string        `mapstructure:"health_url"`
	DomainName string        `mapstructure:"domain_name"`
	AuthURL    string        `mapstructure:"auth_url"`
	TokenURL   string        `mapstructure:"token_url"`
	PublicKey  OidcPublicKey `mapstructure:"public_key"`
	Frontend   OidcClient    `mapstructure:"frontend"`
	Backend    OidcClient    `mapstructure:"backend"`
}

type OidcPublicKey struct {
	StaticKey       string        `mapstructure:"static"`
	JwksURL         string        `mapstructure:"jwks_url"`
	RefreshInterval time.Duration `mapstructure:"refresh_interval"`
	RefreshTimeout  time.Duration `mapstructure:"refresh_timeout"`
}

type OidcClient struct {
	ClientID     string `mapstructure:"client_id"`
	ClientSecret string `mapstructure:"client_secret"`
}
