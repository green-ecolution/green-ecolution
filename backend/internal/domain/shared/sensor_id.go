package shared

import "errors"

var ErrInvalidSensorID = errors.New("sensor ID must not be empty")

type SensorID struct {
	id string
}

func NewSensorID(id string) (SensorID, error) {
	if id == "" {
		return SensorID{}, ErrInvalidSensorID
	}
	return SensorID{id: id}, nil
}

func MustNewSensorID(id string) SensorID {
	s, err := NewSensorID(id)
	if err != nil {
		panic(err)
	}
	return s
}

func (s SensorID) String() string {
	return s.id
}
