package info

import "context"

type InfoRepository interface {
	GetAppInfo(ctx context.Context) (*App, error)
	GetMapInfo(ctx context.Context) (*Map, error)
	GetServerInfo(ctx context.Context) (*Server, error)
	GetServices(ctx context.Context) (*Services, error)
	GetStatistics(ctx context.Context) (*DataStatistics, error)
}
