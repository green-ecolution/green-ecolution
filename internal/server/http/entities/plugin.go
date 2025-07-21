package entities

type PluginResponse struct {
	Slug        string `json:"slug"`
	Name        string `json:"name"`
	Version     string `json:"version"`
	Description string `json:"description"`
	HostPath    string `json:"host_path"`
} //	@name	Plugin

type PluginAuth struct {
	ClientID     string `json:"client_id"`
	ClientSecret string `json:"client_secret"`
} //	@name	PluginAuth

type PluginRegisterRequest struct {
	Slug        string     `json:"slug"`
	Name        string     `json:"name"`
	Description string     `json:"description"`
	Version     string     `json:"version"`
	Path        string     `json:"path"`
	Auth        PluginAuth `json:"auth"`
} //	@name	PluginRegisterRequest

type PluginRegisterResponse struct {
	Success bool                `json:"success"`
	Token   ClientTokenResponse `json:"token"`
} //	@name	PluginRegister

type PluginListResponse struct {
	Plugins []PluginResponse `json:"plugins"`
} //	@name	PluginListResponse
