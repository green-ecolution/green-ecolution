package entities

import "time"

type GeoJSONType string

const (
	FeatureCollection GeoJSONType = "FeatureCollection"
	Feature           GeoJSONType = "Feature"
	LineString        GeoJSONType = "LineString"
)

type GeoJSON struct {
	Type     GeoJSONType      `json:"type"`
	Bbox     []float64        `json:"bbox"`
	Features []GeoJSONFeature `json:"features"`
	Metadata GeoJSONMetadata  `json:"metadata"`
}

type GeoJSONFeature struct {
	Type       GeoJSONType     `json:"type"`
	Bbox       []float64       `json:"bbox"`
	Properties map[string]any  `json:"properties"`
	Geometry   GeoJSONGeometry `json:"geometry"`
}

type GeoJSONGeometry struct {
	Type        GeoJSONType `json:"type"`
	Coordinates [][]float64 `json:"coordinates"`
}

type GeoJSONMetadata struct {
	StartPoint    GeoJSONLocation
	EndPoint      GeoJSONLocation
	WateringPoint GeoJSONLocation
}

type GeoJSONLocation struct {
	Latitude  float64
	Longitude float64
}

type RouteMetadata struct {
	Distance float64
	Refills  int32
	Time     time.Duration
}
