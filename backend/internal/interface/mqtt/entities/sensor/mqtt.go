package sensor

type WatermarkResponse struct {
	Resistance int `json:"resistance"`
	Centibar   int `json:"centibar"`
	Depth      int `json:"depth"`
} //	@Name	Watermark

type MqttPayloadResponse struct {
	Device      string              `json:"device"`
	Battery     float64             `json:"battery"`
	Humidity    float64             `json:"humidity"`
	Temperature float64             `json:"temperature"`
	Latitude    float64             `json:"latitude"`
	Longitude   float64             `json:"longitude"`
	Watermarks  []WatermarkResponse `json:"watermarks"`
} //	@Name	MqttPayload
