package shared

import (
	"errors"
	"fmt"
)

type ErrEntityNotFound string

func (e ErrEntityNotFound) Error() string {
	return fmt.Sprintf("entity not found: %s", string(e))
}

var (
	ErrUnknownError     = errors.New("unknown error")
	ErrTooManyRows      = errors.New("receive more rows then expected")
	ErrConnectionClosed = errors.New("connection is closed")
	ErrTxClosed         = errors.New("transaction closed")
	ErrTxCommitRollback = errors.New("transaction cannot commit or rollback")
	ErrBucketNotExists  = errors.New("bucket don't exists")

	ErrPaginationValueInvalid = errors.New("pagination values are invalid")
	ErrInvalidMapConfig       = errors.New("map configuration not valid")

	ErrS3ServiceDisabled      = errors.New("s3 service is disabled")
	ErrAuthServiceDisabled    = errors.New("auth service is disabled")
	ErrRoutingServiceDisabled = errors.New("routing service is disabled")
)
