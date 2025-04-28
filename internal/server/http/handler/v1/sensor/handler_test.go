package sensor_test

import (
	"context"
	"errors"
	"net/http"
	"testing"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/entities"
	serverEntities "github.com/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/backend/internal/server/http/handler/v1/sensor"
	"github.com/green-ecolution/backend/internal/server/http/middleware"
	"github.com/green-ecolution/backend/internal/service"
	serviceMock "github.com/green-ecolution/backend/internal/service/_mock"
	"github.com/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestGetAllSensors(t *testing.T) {
	t.Run("should return all sensors successfully with default pagination values", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensors(mockSensorService)
		app.Get("/v1/sensor", handler)

		mockSensorService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return(TestSensorList, int64(len(TestSensorList)), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor", nil)
		resp, err := app.Test(req, -1)
		assert.Nil(t, err)
		defer resp.Body.Close()

		// then
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.SensorListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Len(t, response.Data, len(TestSensorList))
		assert.Equal(t, TestSensorList[0].ID, response.Data[0].ID)

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return all sensors successfully with limit 1 and offset 0", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensors(mockSensorService)
		app.Get("/v1/sensor", handler)

		mockSensorService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return(TestSensorList, int64(len(TestSensorList)), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor?page=1&limit=1", nil)
		resp, err := app.Test(req, -1)
		assert.Nil(t, err)
		defer resp.Body.Close()

		// then
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.SensorListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Len(t, response.Data, len(TestSensorList))
		assert.Equal(t, TestSensorList[0].ID, response.Data[0].ID)

		// assert pagination
		assert.Equal(t, int32(1), response.Pagination.CurrentPage)
		assert.Equal(t, int64(len(TestSensorList)), response.Pagination.Total)
		assert.Equal(t, int32(2), *response.Pagination.NextPage)
		assert.Empty(t, response.Pagination.PrevPage)
		assert.Equal(t, int32((len(TestSensorList))/1), response.Pagination.TotalPages)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return error when page is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensors(mockSensorService)
		app.Get("/v1/sensor", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor?page=0&limit=1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return error when limit is invalid", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensors(mockSensorService)
		app.Get("/v1/sensor", handler)

		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor?page=1&limit=0", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return all sensors successfully with full MqttPayload and provider", func(t *testing.T) {
		mockSensorService := serviceMock.NewMockSensorService(t)
		app := fiber.New()
		handler := sensor.GetAllSensors(mockSensorService)

		mockSensorService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{Provider: "test-provider"},
		).Return(TestSensorList, int64(len(TestSensorList)), nil)

		app.Get("/v1/sensor", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor", nil)
		query := req.URL.Query()
		query.Add("provider", "test-provider")
		req.URL.RawQuery = query.Encode()

		resp, err := app.Test(req, -1)
		assert.Nil(t, err)
		defer resp.Body.Close()

		// then
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.SensorListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// Assert response matches test data
		assert.Len(t, response.Data, len(TestSensorList))
		assert.Equal(t, TestSensorList[0].ID, response.Data[0].ID)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return empty sensor list when no sensors found", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensors(mockSensorService)
		app.Get("/v1/sensor", handler)

		mockSensorService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return([]*entities.Sensor{}, int64(0), nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.SensorListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Len(t, response.Data, 0)

		// assert pagination
		assert.Empty(t, response.Pagination)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return 500 when service returns an error", func(t *testing.T) {
		app := fiber.New()
		app.Use(middleware.PaginationMiddleware())
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensors(mockSensorService)
		app.Get("/v1/sensor", handler)

		mockSensorService.EXPECT().GetAll(
			mock.Anything,
			entities.Query{},
		).Return(nil, int64(0), errors.New("service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})
}

func TestGetAllSensorDataById(t *testing.T) {

	t.Run("should return all sensor data successfully", func(t *testing.T) {
		app := fiber.New()
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensorDataByID(mockSensorService)
		app.Get("/v1/sensor/data/:id", handler)

		mockSensorService.EXPECT().GetAllDataByID(
			mock.Anything,
			"sensor-1",
		).Return([]*entities.SensorData{TestSensorData}, nil)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor/data/sensor-1", nil)
		resp, err := app.Test(req, -1)
		assert.Nil(t, err)
		defer resp.Body.Close()

		// then
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.SensorListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		// assert data
		assert.Len(t, response.Data, 1)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return error when no sensor data is found", func(t *testing.T) {
		app := fiber.New()
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensorDataByID(mockSensorService)
		app.Get("/v1/sensor/data/:id", handler)

		mockSensorService.EXPECT().GetAllDataByID(
			mock.Anything,
			"sensor-999",
		).Return(nil, service.NewError(service.NotFound, "not found"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor/data/sensor-999", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return 500 when service returns an error", func(t *testing.T) {
		app := fiber.New()
		mockSensorService := serviceMock.NewMockSensorService(t)
		handler := sensor.GetAllSensorDataByID(mockSensorService)
		app.Get("/v1/sensor/data/:id", handler)

		mockSensorService.EXPECT().GetAllDataByID(
			mock.Anything,
			"sensor-1",
		).Return(nil, errors.New("service error"))

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor/data/sensor-1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})
}

func TestGetSensorById(t *testing.T) {
	t.Run("should return sensor by id successfully", func(t *testing.T) {
		mockSensorService := serviceMock.NewMockSensorService(t)
		app := fiber.New()
		handler := sensor.GetSensorByID(mockSensorService)

		mockSensorService.EXPECT().GetByID(
			mock.Anything,
			"sensor-1",
		).Return(TestSensor, nil)

		app.Get("/v1/sensor/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor/sensor-1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response serverEntities.SensorResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)

		assert.Equal(t, response.ID, TestSensor.ID)
		assert.WithinDuration(t, response.CreatedAt, TestSensor.CreatedAt, time.Second)
		assert.WithinDuration(t, response.UpdatedAt, TestSensor.UpdatedAt, time.Second)
		assert.Equal(t, entities.SensorStatus(response.Status), TestSensor.Status)

		// assert latest data
		assert.Equal(t, response.LatestData.Battery, TestSensorList[0].LatestData.Data.Battery)
		assert.Equal(t, response.LatestData.Humidity, TestSensorList[0].LatestData.Data.Humidity)
		assert.Equal(t, response.LatestData.Temperature, TestSensorList[0].LatestData.Data.Temperature)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return 404 Not Found if sensor does not exist", func(t *testing.T) {
		mockSensorService := serviceMock.NewMockSensorService(t)
		app := fiber.New()
		handler := sensor.GetSensorByID(mockSensorService)

		mockSensorService.EXPECT().GetByID(
			mock.Anything,
			"sensor-999",
		).Return(nil, service.NewError(service.NotFound, "not found"))

		app.Get("/v1/sensor/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor/sensor-999", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return 500 Internal Server Error for service failure", func(t *testing.T) {
		mockSensorService := serviceMock.NewMockSensorService(t)
		app := fiber.New()
		handler := sensor.GetSensorByID(mockSensorService)

		mockSensorService.EXPECT().GetByID(
			mock.Anything,
			"sensor-1",
		).Return(nil, fiber.NewError(fiber.StatusInternalServerError, "service error"))

		app.Get("/v1/sensor/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodGet, "/v1/sensor/sensor-1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})
}

func TestDeleteSensor(t *testing.T) {
	t.Run("should delete sensor successfully", func(t *testing.T) {
		mockSensorService := serviceMock.NewMockSensorService(t)
		app := fiber.New()
		handler := sensor.DeleteSensor(mockSensorService)

		mockSensorService.EXPECT().Delete(
			mock.Anything,
			"sensor-1",
		).Return(nil)

		app.Delete("/v1/sensor/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/sensor/sensor-1", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNoContent, resp.StatusCode)

		mockSensorService.AssertExpectations(t)
	})

	t.Run("should return 404 for non-existing sensor", func(t *testing.T) {
		mockSensorService := serviceMock.NewMockSensorService(t)
		app := fiber.New()
		handler := sensor.DeleteSensor(mockSensorService)

		mockSensorService.EXPECT().Delete(
			mock.Anything,
			"sensor-999",
		).Return(service.NewError(service.NotFound, "not found"))

		app.Delete("/v1/sensor/:id", handler)

		// when
		req, _ := http.NewRequestWithContext(context.Background(), http.MethodDelete, "/v1/sensor/sensor-999", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()
		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusNotFound, resp.StatusCode)
	})
}
