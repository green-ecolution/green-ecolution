package info

import (
	"context"
	"net"
	"net/url"
	"os"
	"runtime"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	versionpkg "github.com/green-ecolution/green-ecolution/backend/internal/storage/version"
)

var version = "development"
var gitCommit = "unknown"
var gitBranch = "develop"
var gitRepository = "https://github.com/green-ecolution/green-ecolution"
var buildTime = ""
var runTime = time.Now()

type InfoRepository struct {
	cfg           *config.Config
	buildTime     time.Time
	gitRepository *url.URL
	mapInfo       entities.Map
	versionRepo   versionpkg.VersionRepository
}

func init() {
	if buildTime == "" || buildTime == "unknown" || buildTime == "now" {
		buildTime = time.Now().Format(time.RFC3339)
	}
}

func NewInfoRepository(cfg *config.Config, versionRepo versionpkg.VersionRepository) (*InfoRepository, error) {
	gitRepository, err := getGitRepository()
	if err != nil {
		return nil, err
	}

	buildTime, err := getBuildTime()
	if err != nil {
		return nil, err
	}

	mapInfo, err := getMapInfo(cfg)
	if err != nil {
		return nil, err
	}

	return &InfoRepository{
		cfg:           cfg,
		buildTime:     buildTime,
		gitRepository: gitRepository,
		mapInfo:       mapInfo,
		versionRepo:   versionRepo,
	}, nil
}

func (r *InfoRepository) GetAppInfo(ctx context.Context) (*entities.App, error) {
	log := logger.GetLogger(ctx)
	hostname, err := r.getHostname()
	if err != nil {
		log.Error("failed to get hostname from host", "error", err)
		return nil, storage.ErrHostnameNotFound
	}

	appURL, err := r.getAppURL()
	if err != nil {
		log.Error("failed to parse configured app url", "error", err, "app_url", r.cfg.Server.AppURL)
		return nil, storage.ErrCannotGetAppURL
	}

	localIP, err := getIP(ctx)
	if err != nil {
		return nil, err
	}

	localInterface, err := getInterface(localIP)
	if err != nil {
		return nil, err
	}

	return &entities.App{
		Version:     version,
		VersionInfo: r.getVersionInfo(ctx),
		GoVersion:   r.getGoVersion(),
		BuildTime:   r.buildTime,
		Git: entities.Git{
			Branch:     gitBranch,
			Commit:     gitCommit,
			Repository: r.gitRepository,
		},
		Server: entities.Server{
			OS:        r.getOS(),
			Arch:      r.getArch(),
			Hostname:  hostname,
			URL:       appURL,
			IP:        localIP,
			Port:      r.getPort(),
			Interface: localInterface,
			Uptime:    r.getUptime(),
		},
		Map:      r.mapInfo,
		Services: r.getServices(),
	}, nil
}

func (r *InfoRepository) getServices() entities.Services {
	vroomEnabled := r.cfg.Routing.Enable && r.cfg.Routing.Valhalla.Optimization.Vroom.Host != ""

	services := []entities.ServiceStatus{
		{
			Name:    "database",
			Enabled: true,
			Healthy: true,
			Message: "Verbunden",
		},
		{
			Name:    "auth",
			Enabled: r.cfg.IdentityAuth.Enable,
			Healthy: r.cfg.IdentityAuth.Enable,
			Message: getServiceMessage(r.cfg.IdentityAuth.Enable),
		},
		{
			Name:    "mqtt",
			Enabled: r.cfg.MQTT.Enable,
			Healthy: r.cfg.MQTT.Enable,
			Message: getServiceMessage(r.cfg.MQTT.Enable),
		},
		{
			Name:    "s3",
			Enabled: r.cfg.S3.Enable,
			Healthy: r.cfg.S3.Enable,
			Message: getServiceMessage(r.cfg.S3.Enable),
		},
		{
			Name:    "routing",
			Enabled: r.cfg.Routing.Enable,
			Healthy: r.cfg.Routing.Enable,
			Message: getServiceMessage(r.cfg.Routing.Enable),
		},
		{
			Name:    "vroom",
			Enabled: vroomEnabled,
			Healthy: vroomEnabled,
			Message: getServiceMessage(vroomEnabled),
		},
	}

	return entities.Services{Items: services}
}

func getServiceMessage(enabled bool) string {
	if enabled {
		return "Aktiviert"
	}
	return "Deaktiviert"
}

func (r *InfoRepository) getVersionInfo(ctx context.Context) entities.VersionInfo {
	if r.versionRepo == nil {
		return versionpkg.CompareVersions(version, "")
	}

	latestInfo, err := r.versionRepo.GetLatestVersion(ctx)
	if err != nil {
		return versionpkg.CompareVersions(version, "")
	}

	result := versionpkg.CompareVersions(version, latestInfo.Version)
	result.ReleaseURL = latestInfo.ReleaseURL
	return result
}

func getMapInfo(cfg *config.Config) (entities.Map, error) {
	if len(cfg.Map.Center) != 2 || len(cfg.Map.BBox) != 4 {
		return entities.Map{}, storage.ErrInvalidMapConfig
	}

	return entities.Map{
		Center: cfg.Map.Center,
		BBox:   cfg.Map.BBox,
	}, nil
}

func (r *InfoRepository) getOS() string {
	return runtime.GOOS
}

func (r *InfoRepository) getHostname() (string, error) {
	return os.Hostname()
}

func (r *InfoRepository) getPort() int {
	return r.cfg.Server.Port
}

func (r *InfoRepository) getAppURL() (*url.URL, error) {
	return url.Parse(r.cfg.Server.AppURL)
}

func (r *InfoRepository) getUptime() time.Duration {
	return time.Since(runTime)
}

func (r *InfoRepository) getGoVersion() string {
	return runtime.Version()
}

func (r *InfoRepository) getArch() string {
	return runtime.GOARCH
}

func getBuildTime() (time.Time, error) {
	return time.Parse(time.RFC3339, buildTime)
}

func getGitRepository() (*url.URL, error) {
	return url.Parse(gitRepository)
}

func getIP(ctx context.Context) (net.IP, error) {
	var d net.Dialer
	conn, err := d.DialContext(ctx, "udp", "green-ecolution.de:80")
	if err != nil {
		return nil, storage.ErrIPNotFound
	}

	defer conn.Close()

	return conn.LocalAddr().(*net.UDPAddr).IP, nil
}

func getInterface(localIP net.IP) (string, error) {
	ifaces, err := net.Interfaces()
	if err != nil {
		return "", storage.ErrIFacesNotFound
	}

	for _, iface := range ifaces {
		address, err := iface.Addrs()
		if err != nil {
			return "", storage.ErrIFacesAddressNotFound
		}

		for _, addr := range address {
			if addr.(*net.IPNet).IP.String() == localIP.String() {
				return iface.Name, nil
			}
		}
	}

	return "", storage.ErrIFacesNotFound
}
