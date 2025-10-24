package mqtt

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"

	MQTT "github.com/eclipse/paho.mqtt.golang"
	"github.com/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/backend/internal/server/mqtt/entities/sensor"
	"github.com/green-ecolution/backend/internal/server/mqtt/entities/sensor/generated"
	"github.com/green-ecolution/backend/internal/service"
)

var (
	ErrCastValue = errors.New("failed to cast mqtt payload")
)

type Mqtt struct {
	cfg    *config.Config
	svc    *service.Services
	mapper sensor.MqttMqttMapper
}

func NewMqtt(cfg *config.Config, services *service.Services) *Mqtt {
	return &Mqtt{
		cfg:    cfg,
		svc:    services,
		mapper: &generated.MqttMqttMapperImpl{},
	}
}

func (m *Mqtt) RunSubscriber(ctx context.Context) {
	opts := MQTT.NewClientOptions()
	opts.AddBroker(m.cfg.MQTT.Broker)
	opts.SetClientID(m.cfg.MQTT.ClientID)
	opts.SetUsername(m.cfg.MQTT.Username)
	opts.SetPassword(m.cfg.MQTT.Password)

	opts.OnConnect = func(_ MQTT.Client) {
		slog.Info("connected to mqtt broker")
	}
	opts.OnConnectionLost = func(_ MQTT.Client, err error) {
		slog.Error("lost connection to mqtt broker", "error", err)
	}

	client := MQTT.NewClient(opts)
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		slog.Error("error connecting to mqtt broker", "error", token.Error())
		return
	}

	token := client.Subscribe(m.cfg.MQTT.Topic, 1, m.handleMqttMessage)
	go func(token MQTT.Token) {
		_ = token.Wait()
		if token.Error() != nil {
			slog.Error("error while subscribing to mqtt broker", "error", token.Error())
		}
	}(token)

	<-ctx.Done()
	slog.Info("shutting down mqtt subscriber")
}

func (m *Mqtt) handleMqttMessage(_ MQTT.Client, msg MQTT.Message) {
	sensorData, err := m.convertToMqttPayloadResponse(msg)
	if err != nil {
		slog.Error("error while converting mqtt payload to sensor data", "error", err)
		return
	}

	slog.Info("received sensor data", "sensor_id", sensorData.Device)
	slog.Debug("detailed sensor data", "sensor_raw_data", fmt.Sprintf("%+v", sensorData))

	domainPayload := m.mapper.FromResponse(sensorData)
	_, err = m.svc.SensorService.HandleMessage(context.Background(), domainPayload)
	if err != nil {
		return
	}
}

func saveCastPayload[T any](raw map[string]any, key string) (T, bool) {
	v, ok := raw[key].(T)
	if !ok {
		slog.Debug("failed to cast value in payload", "key", key, "raw", raw)
		return *new(T), false
	}

	return v, true
}

//nolint:gocyclo // quick and dirty safe cast for the user tests tomorrow, I want to go to bed
func (m *Mqtt) convertToMqttPayloadResponse(msg MQTT.Message) (*sensor.MqttPayloadResponse, error) {
	var raw map[string]any
	if err := json.Unmarshal(msg.Payload(), &raw); err != nil {
		return nil, fmt.Errorf("error unmarshalling json: %w", err)
	}

	uplinkMessage, ok := saveCastPayload[map[string]any](raw, "uplink_message")
	if !ok {
		return nil, ErrCastValue
	}

	decodedPayload, ok := saveCastPayload[map[string]any](uplinkMessage, "decoded_payload")
	if !ok {
		return nil, ErrCastValue
	}

	deviceName, ok := saveCastPayload[string](decodedPayload, "deviceName")
	if !ok {
		return nil, ErrCastValue
	}

	batteryVoltage, ok := saveCastPayload[float64](decodedPayload, "batteryVoltage")
	if !ok {
		return nil, ErrCastValue
	}

	waterContent, ok := saveCastPayload[float64](decodedPayload, "waterContent")
	if !ok {
		return nil, ErrCastValue
	}

	temperature, ok := saveCastPayload[float64](decodedPayload, "temperature")
	if !ok {
		return nil, ErrCastValue
	}

	latitude, ok := saveCastPayload[float64](decodedPayload, "latitude")
	if !ok {
		return nil, ErrCastValue
	}

	longitude, ok := saveCastPayload[float64](decodedPayload, "longitude")
	if !ok {
		return nil, ErrCastValue
	}

	wm30Res, ok := saveCastPayload[float64](decodedPayload, "WM30_Resistance")
	if !ok {
		return nil, ErrCastValue
	}

	wm30Cb, ok := saveCastPayload[float64](decodedPayload, "WM30_CB")
	if !ok {
		return nil, ErrCastValue
	}

	wm60Res, ok := saveCastPayload[float64](decodedPayload, "WM60_Resistance")
	if !ok {
		return nil, ErrCastValue
	}

	wm60Cb, ok := saveCastPayload[float64](decodedPayload, "WM60_CB")
	if !ok {
		return nil, ErrCastValue
	}

	wm90Res, ok := saveCastPayload[float64](decodedPayload, "WM90_Resistance")
	if !ok {
		return nil, ErrCastValue
	}

	wm90Cb, ok := saveCastPayload[float64](decodedPayload, "WM90_CB")
	if !ok {
		return nil, ErrCastValue
	}

	payload := &sensor.MqttPayloadResponse{
		Device:      deviceName,
		Battery:     batteryVoltage,
		Humidity:    waterContent,
		Temperature: temperature,
		Latitude:    latitude,
		Longitude:   longitude,
		Watermarks: []sensor.WatermarkResponse{
			{
				Resistance: int(wm30Res),
				Centibar:   int(wm30Cb),
				Depth:      30,
			},
			{
				Resistance: int(wm60Res),
				Centibar:   int(wm60Cb),
				Depth:      60,
			},
			{
				Resistance: int(wm90Res),
				Centibar:   int(wm90Cb),
				Depth:      90,
			},
		},
	}

	return payload, nil
}
