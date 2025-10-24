package s3

import (
	"io"

	"github.com/green-ecolution/backend/internal/storage"
	"golang.org/x/net/context"
)

// S3DummyRepo is used to disable the S3 service by configuration
type S3DummyRepo struct {
}

func NewS3DummyRepo() *S3DummyRepo {
	return &S3DummyRepo{}
}

func (s *S3DummyRepo) BucketExists(_ context.Context) (bool, error) {
	return true, nil
}

func (s *S3DummyRepo) PutObject(_ context.Context, _, _ string, _ int64, _ io.Reader) error {
	return nil
}

func (s *S3DummyRepo) GetObject(_ context.Context, _ string) (io.ReadSeekCloser, error) {
	return nil, storage.ErrS3ServiceDisabled
}
