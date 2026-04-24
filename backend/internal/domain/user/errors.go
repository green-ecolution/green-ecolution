package user

import "errors"

var (
	ErrNotFound           = errors.New("user not found")
	ErrNotCorrectRole     = errors.New("user has an incorrect role")
	ErrNotMatchingLicense = errors.New("user has no matching driving license")
)
