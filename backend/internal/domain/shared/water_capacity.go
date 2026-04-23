package shared

import (
	"errors"
	"fmt"
)

var ErrInvalidWaterCapacity = errors.New("water capacity must be >= 0")

type WaterCapacity struct {
	liters float64
}

func NewWaterCapacity(liters float64) (WaterCapacity, error) {
	if liters < 0 {
		return WaterCapacity{}, ErrInvalidWaterCapacity
	}
	return WaterCapacity{liters: liters}, nil
}

func MustNewWaterCapacity(liters float64) WaterCapacity {
	w, err := NewWaterCapacity(liters)
	if err != nil {
		panic(err)
	}
	return w
}

func (w WaterCapacity) Liters() float64 {
	return w.liters
}

func (w WaterCapacity) Add(other WaterCapacity) WaterCapacity {
	return WaterCapacity{liters: w.liters + other.liters}
}

func (w WaterCapacity) String() string {
	return fmt.Sprintf("%.2fL", w.liters)
}
