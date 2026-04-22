package entities

import (
	"errors"
	"time"
)

var ErrInvalidPlantingYear = errors.New("planting year must be > 0 and not in the future")

type PlantingYear struct {
	year int32
}

func NewPlantingYear(year int32) (PlantingYear, error) {
	if year <= 0 || year > int32(time.Now().Year()) {
		return PlantingYear{}, ErrInvalidPlantingYear
	}
	return PlantingYear{year: year}, nil
}

func MustNewPlantingYear(year int32) PlantingYear {
	p, err := NewPlantingYear(year)
	if err != nil {
		panic(err)
	}
	return p
}

func (p PlantingYear) Value() int32 {
	return p.year
}
