package config

import (
	"flag"
	"log/slog"
	"path"
	"reflect"
	"strconv"
	"strings"

	"github.com/go-viper/mapstructure/v2"
	"github.com/spf13/viper"
)

func InitViper() (*Config, error) {
	configPath := flag.String("config", "./config/config.yaml", "path to configuratione file")
	flag.Parse()

	configName := path.Base(*configPath)
	configDir := path.Dir(*configPath)
	configType := path.Ext(*configPath)
	if len(configType) < 1 {
		configType = configType[1:]
	} else {
		configType = "yaml"
	}

	viper.SetConfigName(configName)
	viper.SetConfigType(configType)
	viper.AddConfigPath(configDir)
	viper.SetEnvPrefix("GE")
	viper.AutomaticEnv()
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_", "-", "_"))

	setDefaults()

	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); !ok {
			return nil, err
		}
		slog.Info("No config file found, using environment variables only")
	}

	var cfg Config
	if err := viper.Unmarshal(&cfg, viper.DecodeHook(
		mapstructure.ComposeDecodeHookFunc(
			mapstructure.StringToTimeDurationHookFunc(),
			stringToFloat64SliceHookFunc(),
			mapstructure.StringToSliceHookFunc(","),
		),
	)); err != nil {
		slog.Error("Error unmarshalling config", "error", err)
		return nil, err
	}

	return &cfg, nil
}

func stringToFloat64SliceHookFunc() mapstructure.DecodeHookFunc {
	return func(f reflect.Type, t reflect.Type, data interface{}) (interface{}, error) {
		if f.Kind() != reflect.String {
			return data, nil
		}
		if t != reflect.TypeOf([]float64{}) {
			return data, nil
		}

		str := data.(string)
		if str == "" {
			return []float64{}, nil
		}

		parts := strings.Split(str, ",")
		result := make([]float64, len(parts))
		for i, part := range parts {
			val, err := strconv.ParseFloat(strings.TrimSpace(part), 64)
			if err != nil {
				return nil, err
			}
			result[i] = val
		}
		return result, nil
	}
}

func setDefaults() {
	// Server
	viper.SetDefault("server.port", 3000)
	viper.SetDefault("server.development", false)
	viper.SetDefault("server.app_url", "")
	viper.SetDefault("server.logs.level", "info")
	viper.SetDefault("server.logs.format", "text")
	viper.SetDefault("server.database.host", "")
	viper.SetDefault("server.database.port", 5432)
	viper.SetDefault("server.database.username", "")
	viper.SetDefault("server.database.password", "")
	viper.SetDefault("server.database.name", "")
	viper.SetDefault("server.database.timeout", "30s")

	// Dashboard
	viper.SetDefault("dashboard.title", "Green Ecolution Dashboard")

	// Auth
	viper.SetDefault("auth.enable", true)
	viper.SetDefault("auth.oidc_provider.base_url", "")
	viper.SetDefault("auth.oidc_provider.domain_name", "")
	viper.SetDefault("auth.oidc_provider.auth_url", "")
	viper.SetDefault("auth.oidc_provider.token_url", "")
	viper.SetDefault("auth.oidc_provider.public_key.static", "")
	viper.SetDefault("auth.oidc_provider.frontend.client_id", "")
	viper.SetDefault("auth.oidc_provider.frontend.client_secret", "")
	viper.SetDefault("auth.oidc_provider.backend.client_id", "")
	viper.SetDefault("auth.oidc_provider.backend.client_secret", "")

	// Routing
	viper.SetDefault("routing.enable", true)
	viper.SetDefault("routing.start_point", []float64{})
	viper.SetDefault("routing.end_point", []float64{})
	viper.SetDefault("routing.watering_point", []float64{})
	viper.SetDefault("routing.valhalla.host", "")
	viper.SetDefault("routing.valhalla.optimization.vroom.host", "")
	viper.SetDefault("routing.ors.host", "")
	viper.SetDefault("routing.ors.optimization.vroom.host", "")

	// S3
	viper.SetDefault("s3.enable", true)
	viper.SetDefault("s3.endpoint", "")
	viper.SetDefault("s3.region", "")
	viper.SetDefault("s3.use_ssl", false)
	viper.SetDefault("s3.route-gpx.bucket", "")
	viper.SetDefault("s3.route-gpx.accessKey", "")
	viper.SetDefault("s3.route-gpx.secretAccessKey", "")

	// MQTT
	viper.SetDefault("mqtt.enable", true)
	viper.SetDefault("mqtt.broker", "")
	viper.SetDefault("mqtt.client_id", "")
	viper.SetDefault("mqtt.username", "")
	viper.SetDefault("mqtt.password", "")
	viper.SetDefault("mqtt.topic", "")

	// Map
	viper.SetDefault("map.center", []float64{})
	viper.SetDefault("map.bbox", []float64{})
}
