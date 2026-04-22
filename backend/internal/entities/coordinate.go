package entities

import "errors"

var (
	ErrInvalidLatitude  = errors.New("latitude must be between -90 and 90")
	ErrInvalidLongitude = errors.New("longitude must be between -180 and 180")
)

type Coordinate struct {
	lat float64
	lng float64
}

func NewCoordinate(lat, lng float64) (Coordinate, error) {
	if lat < -90 || lat > 90 {
		return Coordinate{}, ErrInvalidLatitude
	}
	if lng < -180 || lng > 180 {
		return Coordinate{}, ErrInvalidLongitude
	}
	return Coordinate{lat: lat, lng: lng}, nil
}

func MustNewCoordinate(lat, lng float64) Coordinate {
	c, err := NewCoordinate(lat, lng)
	if err != nil {
		panic(err)
	}
	return c
}

func NewCoordinateFromOptional(lat, lng *float64) (*Coordinate, error) {
	if lat == nil || lng == nil {
		return nil, nil
	}
	c, err := NewCoordinate(*lat, *lng)
	if err != nil {
		return nil, err
	}
	return &c, nil
}

func (c Coordinate) Latitude() float64 {
	return c.lat
}

func (c Coordinate) Longitude() float64 {
	return c.lng
}
