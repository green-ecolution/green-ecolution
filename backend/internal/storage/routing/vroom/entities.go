package vroom

type VroomLocation []float64

type VroomVehicle struct {
	ID          int32         `json:"id"`
	Description string        `json:"description"`
	Profile     string        `json:"profile"`
	Start       VroomLocation `json:"start"`
	End         VroomLocation `json:"end"`
	Capacity    []int32       `json:"capacity"`
}

type VroomShipments struct {
	Amount   []int32           `json:"amount"`
	Pickup   VroomShipmentStep `json:"pickup"`
	Delivery VroomShipmentStep `json:"delivery"`
}

type VroomShipmentStep struct {
	ID          int32     `json:"id"`
	Description string    `json:"description"`
	Location    []float64 `json:"location"`
}

type VroomReq struct {
	Vehicles  []VroomVehicle   `json:"vehicles"`
	Shipments []VroomShipments `json:"shipments"`
}

type VroomComputingTime struct {
	Loading int32 `json:"loading"`
	Solving int32 `json:"solving"`
	Routing int32 `json:"routing"`
}

type VroomSummary struct {
	Cost          int32              `json:"cost"`
	Routes        int32              `json:"routes"`
	Unassigend    int32              `json:"unassigend"`
	Delivery      []int32            `json:"delivery"`
	Amount        []int32            `json:"amount"`
	Pickup        []int32            `json:"pickup"`
	Setup         int32              `json:"setup"`
	Service       int32              `json:"service"`
	Duration      int32              `json:"duration"`
	WaitingTime   int32              `json:"waiting_time"`
	ComputingTime VroomComputingTime `json:"computing_time"`
}

type VroomRouteStep struct {
	Type        string        `json:"type"`
	Location    VroomLocation `json:"location"`
	Setup       int32         `json:"setup"`
	Service     int32         `json:"service"`
	WaitingTime int32         `json:"waiting_time"`
	Load        []int32       `json:"load"`
	Arrival     int32         `json:"arrival"`
	Duration    int32         `json:"duration"`
}

type VroomRoutes struct {
	Vehicle     int32            `json:"vehicle"`
	Cost        int32            `json:"cost"`
	Delivery    []int32          `json:"delivery"`
	Amount      []int32          `json:"amount"`
	Pickup      []int32          `json:"pickup"`
	Setup       int32            `json:"setup"`
	Service     int32            `json:"service"`
	Duration    int32            `json:"duration"`
	WaitingTime int32            `json:"waiting_time"`
	Priority    int32            `json:"priority"`
	Steps       []VroomRouteStep `json:"steps"`
}

type VroomResponse struct {
	Code    int32         `json:"code"`
	Error   *string       `json:"error,omitempty"`
	Summary VroomSummary  `json:"summary"`
	Routes  []VroomRoutes `json:"routes"`
}
