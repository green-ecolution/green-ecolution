package info_test

import (
	"net"
	"net/url"
	"time"

	"github.com/green-ecolution/backend/internal/entities"
)

var (
	repoURL, _   = url.Parse("https://github.com/green-ecolution/backend")
	serverURL, _ = url.Parse("http://localhost")

	TestInfo = &entities.App{
		Version:   "1.0.0",
		GoVersion: "go1.23.2",
		BuildTime: time.Date(2023, 1, 1, 0, 0, 0, 0, time.UTC),
		Git: entities.Git{
			Branch:     "main",
			Commit:     "abcd1234",
			Repository: repoURL,
		},
		Server: entities.Server{
			OS:        "linux",
			Arch:      "amd64",
			Hostname:  "localhost",
			URL:       serverURL,
			IP:        net.ParseIP("127.0.0.1"),
			Port:      8080,
			Interface: "eth0",
			Uptime:    24 * time.Hour,
		},
	}
)
