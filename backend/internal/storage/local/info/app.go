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
	cfg              *config.Config
	buildTime        time.Time
	gitRepository    *url.URL
	mapInfo          entities.Map
	versionRepo      versionpkg.VersionRepository
	treeRepo         TreeCountRepo
	sensorRepo       SensorCountRepo
	vehicleRepo      VehicleCountRepo
	treeClusterRepo  TreeClusterCountRepo
	wateringPlanRepo WateringPlanCountRepo
	dbPool           DBPool
	s3Repo           S3HealthChecker
}

type TreeCountRepo interface {
	GetCount(ctx context.Context, query entities.TreeQuery) (int64, error)
}

type SensorCountRepo interface {
	GetCount(ctx context.Context, query entities.Query) (int64, error)
}

type VehicleCountRepo interface {
	GetCount(ctx context.Context, query entities.Query) (int64, error)
}

type TreeClusterCountRepo interface {
	GetCount(ctx context.Context, query entities.TreeClusterQuery) (int64, error)
}

type WateringPlanCountRepo interface {
	GetCount(ctx context.Context, query entities.Query) (int64, error)
}

type DBPool interface {
	Ping(ctx context.Context) error
}

func init() {
	if buildTime == "" || buildTime == "unknown" || buildTime == "now" {
		buildTime = time.Now().Format(time.RFC3339)
	}
}

type InfoRepositoryDeps struct {
	TreeRepo         TreeCountRepo
	SensorRepo       SensorCountRepo
	VehicleRepo      VehicleCountRepo
	TreeClusterRepo  TreeClusterCountRepo
	WateringPlanRepo WateringPlanCountRepo
	DBPool           DBPool
	S3Repo           S3HealthChecker
}

func NewInfoRepository(cfg *config.Config, versionRepo versionpkg.VersionRepository, deps *InfoRepositoryDeps) (*InfoRepository, error) {
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

	repo := &InfoRepository{
		cfg:           cfg,
		buildTime:     buildTime,
		gitRepository: gitRepository,
		mapInfo:       mapInfo,
		versionRepo:   versionRepo,
	}

	if deps != nil {
		repo.treeRepo = deps.TreeRepo
		repo.sensorRepo = deps.SensorRepo
		repo.vehicleRepo = deps.VehicleRepo
		repo.treeClusterRepo = deps.TreeClusterRepo
		repo.wateringPlanRepo = deps.WateringPlanRepo
		repo.dbPool = deps.DBPool
		repo.s3Repo = deps.S3Repo
	}

	return repo, nil
}

func (r *InfoRepository) SetDependencies(deps *InfoRepositoryDeps) {
	if deps == nil {
		return
	}
	r.treeRepo = deps.TreeRepo
	r.sensorRepo = deps.SensorRepo
	r.vehicleRepo = deps.VehicleRepo
	r.treeClusterRepo = deps.TreeClusterRepo
	r.wateringPlanRepo = deps.WateringPlanRepo
	r.dbPool = deps.DBPool
	r.s3Repo = deps.S3Repo
}

func (r *InfoRepository) GetAppInfo(ctx context.Context) (*entities.App, error) {
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
		Map: r.mapInfo,
	}, nil
}

func (r *InfoRepository) GetMapInfo(_ context.Context) (*entities.Map, error) {
	return &r.mapInfo, nil
}

func (r *InfoRepository) GetServerInfo(ctx context.Context) (*entities.Server, error) {
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

	return &entities.Server{
		OS:        r.getOS(),
		Arch:      r.getArch(),
		Hostname:  hostname,
		URL:       appURL,
		IP:        localIP,
		Port:      r.getPort(),
		Interface: localInterface,
		Uptime:    r.getUptime(),
	}, nil
}

func (r *InfoRepository) GetServices(ctx context.Context) (*entities.Services, error) {
	services := r.getServices(ctx)
	return &services, nil
}

func (r *InfoRepository) GetStatistics(ctx context.Context) (*entities.DataStatistics, error) {
	stats := &entities.DataStatistics{}

	if r.treeRepo != nil {
		count, err := r.treeRepo.GetCount(ctx, entities.TreeQuery{})
		if err == nil {
			stats.TreeCount = count
		}
	}

	if r.sensorRepo != nil {
		count, err := r.sensorRepo.GetCount(ctx, entities.Query{})
		if err == nil {
			stats.SensorCount = count
		}
	}

	if r.vehicleRepo != nil {
		count, err := r.vehicleRepo.GetCount(ctx, entities.Query{})
		if err == nil {
			stats.VehicleCount = count
		}
	}

	if r.treeClusterRepo != nil {
		count, err := r.treeClusterRepo.GetCount(ctx, entities.TreeClusterQuery{})
		if err == nil {
			stats.TreeClusterCount = count
		}
	}

	if r.wateringPlanRepo != nil {
		count, err := r.wateringPlanRepo.GetCount(ctx, entities.Query{})
		if err == nil {
			stats.WateringPlanCount = count
		}
	}

	return stats, nil
}

func (r *InfoRepository) getServices(ctx context.Context) entities.Services {
	checker := newHealthChecker(r)
	services := checker.checkAll(ctx)
	return entities.Services{Items: services}
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
