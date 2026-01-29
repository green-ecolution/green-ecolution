package entities

type AppInfoResponse struct {
	Version   string         `json:"version"`
	BuildTime string         `json:"buildTime"`
	GoVersion string         `json:"goVersion"`
	Git       GitResponse    `json:"git"`
	Server    ServerResponse `json:"server,omitzero"`
	Map       MapResponse    `json:"map"`
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
