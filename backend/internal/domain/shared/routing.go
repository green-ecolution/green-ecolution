package entities

import "time"

type GeoJSONType string // @name GeoJSONType

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
} // @name GeoJSON

type GeoJSONFeature struct {
	Type       GeoJSONType     `json:"type"`
	Bbox       []float64       `json:"bbox"`
	Properties map[string]any  `json:"properties"`
	Geometry   GeoJSONGeometry `json:"geometry"`
} // @name GeoJSONFeature

type GeoJSONGeometry struct {
	Type        GeoJSONType `json:"type"`
	Coordinates [][]float64 `json:"coordinates"`
} // @name GeoJSONGeometry

type GeoJSONMetadata struct {
	StartPoint    GeoJSONLocation
	EndPoint      GeoJSONLocation
	WateringPoint GeoJSONLocation
} // @name GeoJSONMetadata

type GeoJSONLocation struct {
	Coordinate Coordinate
} // @name GeoJSONLocation

type RouteMetadata struct {
	Distance Distance
	Refills  int32
	Time     time.Duration
}
