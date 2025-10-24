package s3

import (
	"io"
	"log/slog"

	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
	"golang.org/x/net/context"
)

type S3RepoCfg struct {
	bucketName      string
	endpoint        string
	region          string
	accessKeyID     string
	secretAccessKey string
	useSSL          bool
}

type S3Repository struct {
	cfg    *S3RepoCfg
	client *minio.Client
}

func NewS3Repository(cfg *S3RepoCfg) (*S3Repository, error) {
	client, err := minio.New(cfg.endpoint, &minio.Options{
		Creds:  credentials.NewStaticV4(cfg.accessKeyID, cfg.secretAccessKey, ""),
		Secure: cfg.useSSL,
		Region: cfg.region,
	})
	if err != nil {
		slog.Error("failed to create bucket client", "error", err)
		return nil, err
	}

	return &S3Repository{
		cfg:    cfg,
		client: client,
	}, nil
}

func (s *S3Repository) BucketExists(ctx context.Context) (bool, error) {
	return s.client.BucketExists(ctx, s.cfg.bucketName)
}

func (s *S3Repository) PutObject(ctx context.Context, objName, contentType string, contentLength int64, r io.Reader) error {
	_, err := s.client.PutObject(ctx, s.cfg.bucketName, objName, r, contentLength, minio.PutObjectOptions{ContentType: contentType})
	return err
}

func (s *S3Repository) GetObject(ctx context.Context, objName string) (io.ReadSeekCloser, error) {
	return s.client.GetObject(ctx, s.cfg.bucketName, objName, minio.GetObjectOptions{})
}
