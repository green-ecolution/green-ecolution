package s3

import (
	"context"
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
)

func NewRepository(cfg *config.Config) (*storage.Repository, error) {
	slog.Info("creating s3 repository", "bucket_name", cfg.S3.RouteGpx.Bucket, "endpoint", cfg.S3.Endpoint, "region", cfg.S3.Region, "use_ssl", cfg.S3.UseSSL)
	gpxBucket, err := NewS3Repository(&S3RepoCfg{
		bucketName:      cfg.S3.RouteGpx.Bucket,
		endpoint:        cfg.S3.Endpoint,
		region:          cfg.S3.Region,
		accessKeyID:     cfg.S3.RouteGpx.AccessKey,
		secretAccessKey: cfg.S3.RouteGpx.SecretAccessKey,
		useSSL:          cfg.S3.UseSSL,
	})
	if err != nil {
		return nil, err
	}

	bucketExists, err := gpxBucket.BucketExists(context.Background())
	if err != nil || !bucketExists {
		slog.Error("bucket don't exists", "error", err, "bucket_name", gpxBucket.cfg.bucketName)
		return nil, storage.ErrBucketNotExists
	}

	slog.Info("successfully initialized s3 repository", "bucket_name", gpxBucket.cfg.bucketName)
	return &storage.Repository{
		GpxBucket: gpxBucket,
	}, nil
}
