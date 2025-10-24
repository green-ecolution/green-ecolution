package entities

import "time"

type Region struct {
	ID        int32
	CreatedAt time.Time
	UpdatedAt time.Time
	Name      string
}
