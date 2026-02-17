package entities

import (
	"net"
	"net/url"
	"time"
)

type ServiceStatus struct {
	Name         string
	Enabled      bool
	Healthy      bool
	ResponseTime time.Duration
	LastChecked  time.Time
	Message      string
}

type DataStatistics struct {
	TreeCount         int64
	SensorCount       int64
	VehicleCount      int64
	TreeClusterCount  int64
	WateringPlanCount int64
}

type Services struct {
	Items []ServiceStatus
}

type VersionInfo struct {
	Current         string
	Latest          string
	UpdateAvailable bool
	IsDevelopment   bool
	IsStage         bool
	ReleaseURL      string
}

type App struct {
	Version     string
	VersionInfo VersionInfo
	GoVersion   string
	BuildTime   time.Time
	Git         Git
	Server      Server
	Map         Map
	Services    Services
}

type Git struct {
	Branch     string
	Commit     string
	Repository *url.URL
}

type Server struct {
	OS        string
	Arch      string
	Hostname  string
	URL       *url.URL
	IP        net.IP
	Port      int
	Interface string
	Uptime    time.Duration
}

type Map struct {
	Center []float64
	BBox   []float64
}
