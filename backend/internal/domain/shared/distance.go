package shared

import (
	"errors"
	"fmt"
)

var ErrInvalidDistance = errors.New("distance must be >= 0")

type Distance struct {
	meters float64
}

func NewDistance(meters float64) (Distance, error) {
	if meters < 0 {
		return Distance{}, ErrInvalidDistance
	}
	return Distance{meters: meters}, nil
}

func MustNewDistance(meters float64) Distance {
	d, err := NewDistance(meters)
	if err != nil {
		panic(err)
	}
	return d
}

func (d Distance) Meters() float64 {
	return d.meters
}

func (d Distance) String() string {
	return fmt.Sprintf("%.2fm", d.meters)
}
