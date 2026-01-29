package entities

type ServiceStatusResponse struct {
	Name    string `json:"name"`
	Enabled bool   `json:"enabled"`
	Healthy bool   `json:"healthy"`
	Message string `json:"message,omitempty"`
} //	@Name	ServiceStatus

type ServicesResponse struct {
	Items []ServiceStatusResponse `json:"items"`
} //	@Name	ServicesInfo

type VersionInfoResponse struct {
	Current         string `json:"current"`
	Latest          string `json:"latest,omitempty"`
	UpdateAvailable bool   `json:"updateAvailable"`
	IsDevelopment   bool   `json:"isDevelopment"`
	IsStage         bool   `json:"isStage"`
	ReleaseURL      string `json:"releaseUrl,omitempty"`
} //	@Name	VersionInfo

type AppInfoResponse struct {
	Version     string              `json:"version"`
	VersionInfo VersionInfoResponse `json:"versionInfo"`
	BuildTime   string              `json:"buildTime"`
	GoVersion   string              `json:"goVersion"`
	Git         GitResponse         `json:"git"`
	Server      ServerResponse      `json:"server,omitzero"`
	Map         MapResponse         `json:"map"`
	Services    ServicesResponse    `json:"services"`
} //	@Name	AppInfo

type GitResponse struct {
	Branch     string `json:"branch"`
	Commit     string `json:"commit"`
	Repository string `json:"repository"`
} //	@Name	GitInfo

type ServerResponse struct {
	OS        string `json:"os"`
	Arch      string `json:"arch"`
	Hostname  string `json:"hostname"`
	URL       string `json:"url"`
	IP        string `json:"ip"`
	Port      int    `json:"port"`
	Interface string `json:"interface"`
	Uptime    string `json:"uptime"`
} //	@Name	ServerInfo

type MapResponse struct {
	Center []float64 `json:"center"`
	BBox   []float64 `json:"bbox"`
} //	@Name	MapInfo
