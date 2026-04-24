package vehicle

import "errors"

var (
	ErrNotFound    = errors.New("vehicle not found")
	ErrUnknownType = errors.New("unknown vehicle type")
)
