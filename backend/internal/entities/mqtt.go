package entities

type Watermark struct {
	Centibar   int
	Resistance int
	Depth      int
}

type MqttPayload struct {
	Device      string `validate:"required"`
	Battery     float64
	Humidity    float64
	Temperature float64
	Latitude    float64 `validate:"omitempty,min=-90,max=90"`
	Longitude   float64 `validate:"omitempty,min=-180,max=180"`
	Watermarks  []Watermark
}
