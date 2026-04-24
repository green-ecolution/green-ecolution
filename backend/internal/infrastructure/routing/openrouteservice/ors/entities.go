package ors

import "time"

type OrsDirectionRequest struct {
	Coordinates [][]float64 `json:"coordinates"`
	Units       string      `json:"units"`
	Language    string      `json:"language"`
}

type OrsResponse struct {
	BBox     []float64 `json:"bbox"`
	Routes   []Route   `json:"routes"`
	Metadata Metadata  `json:"metadata"`
}

type Route struct {
	Summary   Summary   `json:"summary"`
	Segments  []Segment `json:"segments"`
	BBox      []float64 `json:"bbox"`
	Geometry  string    `json:"geometry"`
	WayPoints []int     `json:"way_points"`
}

type Summary struct {
	Distance float64 `json:"distance"`
	Duration float64 `json:"duration"`
}

type Segment struct {
	Distance float64 `json:"distance"`
	Duration float64 `json:"duration"`
	Steps    []Step  `json:"steps"`
}

type Step struct {
	Distance    float64 `json:"distance"`
	Duration    float64 `json:"duration"`
	Type        int     `json:"type"`
	Instruction string  `json:"instruction"`
	Name        string  `json:"name"`
	WayPoints   []int   `json:"way_points"`
}

type Metadata struct {
	Attribution string        `json:"attribution"`
	Service     string        `json:"service"`
	Timestamp   int64         `json:"timestamp"`
	Query       Query         `json:"query"`
	Engine      EngineDetails `json:"engine"`
}

type Query struct {
	Coordinates [][]float64 `json:"coordinates"`
	Profile     string      `json:"profile"`
	ProfileName string      `json:"profileName"`
	Format      string      `json:"format"`
}

type EngineDetails struct {
	Version   string    `json:"version"`
	BuildDate time.Time `json:"build_date"`
	GraphDate time.Time `json:"graph_date"`
}
