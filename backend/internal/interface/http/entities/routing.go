package entities

type GeoJSONType string //	@Name	GeoJsonType

const (
	FeatureCollection GeoJSONType = "FeatureCollection"
)

type GeoJSON struct {
	Type     GeoJSONType      `json:"type"`
	Bbox     []float64        `json:"bbox"`
	Features []GeoJSONFeature `json:"features"`
	Metadata GeoJSONMetadata  `json:"metadata"`
} //	@Name	GeoJson

type GeoJSONFeature struct {
	Type       GeoJSONType     `json:"type"`
	Bbox       []float64       `json:"bbox"`
	Properties map[string]any  `json:"properties"`
	Geometry   GeoJSONGeometry `json:"geometry"`
} //	@Name	GeoJsonFeature

type GeoJSONGeometry struct {
	Type        GeoJSONType `json:"type"`
	Coordinates [][]float64 `json:"coordinates"`
} //	@Name	GeoJsonGeometry

type GeoJSONMetadata struct {
	StartPoint    GeoJSONLocation `json:"start_point"`
	EndPoint      GeoJSONLocation `json:"end_point"`
	WateringPoint GeoJSONLocation `json:"watering_point"`
} //	@Name	GeoJSONMetadata

type GeoJSONLocation struct {
	Latitude  float64 `json:"latitude"`
	Longitude float64 `json:"longitude"`
} //	@Name	GeoJSONLocation

type RouteRequest struct {
	TransporterID  int32   `json:"transporter_id"`
	TrailerID      *int32  `json:"trailer_id,omitempty" validate:"optional"`
	TreeClusterIDs []int32 `json:"cluster_ids"`
} //	@Name	RouteRequest
