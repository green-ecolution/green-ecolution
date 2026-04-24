package valhalla

type Location struct {
	Lat  float64 `json:"lat"`
	Lon  float64 `json:"lon"`
	Type string  `json:"type"`
}

type CostingOptions struct {
	Width     float64 `json:"width"`
	Height    float64 `json:"height"`
	Length    float64 `json:"length"`
	Weight    float64 `json:"weight"`
	AxleLoad  float64 `json:"axle_load"`
	AxleCount int     `json:"axle_count"`
}

type DirectionRequest struct {
	Locations      []Location                `json:"locations"`
	Costing        string                    `json:"costing"`
	CostingOptions map[string]CostingOptions `json:"costing_options"`
	Units          string                    `json:"units"`
	Language       string                    `json:"language"`
	Format         string                    `json:"format"`
}

type DirectionResponse struct {
	Trip TripResponse `json:"trip"`
}

type TripResponse struct {
	Locations     []LocationResponse `json:"locations"`
	Legs          []LegResponse      `json:"legs"`
	Summary       SummaryResponse    `json:"summary"`
	StatusMessage string             `json:"status_message"`
	Status        int                `json:"status"`
	Units         string             `json:"units"`
	Language      string             `json:"language"`
}

type LocationResponse struct {
	Type          string  `json:"type"`
	Lat           float64 `json:"lat"`
	Lon           float64 `json:"lon"`
	SideOfStreet  string  `json:"side_of_street"`
	OriginalIndex int     `json:"original_index"`
}

type LegResponse struct {
	Maneuvers []ManeuverResponse `json:"maneuvers"`
	Summary   SummaryResponse    `json:"summary"`
	Shape     string             `json:"shape"`
}

type ManeuverResponse struct {
	Type                                int      `json:"type"`
	Instruction                         string   `json:"instruction"`
	VerbalSuccinctTransitionInstruction string   `json:"verbal_succinct_transition_instruction"`
	VerbalPreTransitionInstruction      string   `json:"verbal_pre_transition_instruction"`
	VerbalPostTransitionInstruction     string   `json:"verbal_post_transition_instruction"`
	StreetNames                         []string `json:"street_names"`
	BeginStreetNames                    []string `json:"begin_street_names,omitempty"`
	Time                                float64  `json:"time"`
	Length                              float64  `json:"length"`
	Cost                                float64  `json:"cost"`
	BeginShapeIndex                     int      `json:"begin_shape_index"`
	EndShapeIndex                       int      `json:"end_shape_index"`
	VerbalMultiCue                      bool     `json:"verbal_multi_cue,omitempty"`
	TravelMode                          string   `json:"travel_mode"`
	TravelType                          string   `json:"travel_type"`
}

type SummaryResponse struct {
	HasTimeRestrictions bool    `json:"has_time_restrictions"`
	HasToll             bool    `json:"has_toll"`
	HasHighway          bool    `json:"has_highway"`
	HasFerry            bool    `json:"has_ferry"`
	MinLat              float64 `json:"min_lat"`
	MinLon              float64 `json:"min_lon"`
	MaxLat              float64 `json:"max_lat"`
	MaxLon              float64 `json:"max_lon"`
	Time                float64 `json:"time"`
	Length              float64 `json:"length"`
	Cost                float64 `json:"cost"`
}
