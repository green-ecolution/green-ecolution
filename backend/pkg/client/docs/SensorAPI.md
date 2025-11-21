# \SensorAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**DeleteSensor**](SensorAPI.md#DeleteSensor) | **Delete** /v1/sensor/{sensor_id} | Delete sensor
[**GetAllSensorDataById**](SensorAPI.md#GetAllSensorDataById) | **Get** /v1/sensor/data/{sensor_id} | Get all sensor data by id
[**GetAllSensors**](SensorAPI.md#GetAllSensors) | **Get** /v1/sensor | Get all sensors
[**GetSensorById**](SensorAPI.md#GetSensorById) | **Get** /v1/sensor/{sensor_id} | Get sensor by ID



## DeleteSensor

> DeleteSensor(ctx, sensorId).Execute()

Delete sensor



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
	openapiclient "github.com/green-ecolution/green-ecolution/backend/pkg/client"
)

func main() {
	sensorId := "sensorId_example" // string | Sensor ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	r, err := apiClient.SensorAPI.DeleteSensor(context.Background(), sensorId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `SensorAPI.DeleteSensor``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**sensorId** | **string** | Sensor ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiDeleteSensorRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

 (empty response body)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetAllSensorDataById

> SensorDataList GetAllSensorDataById(ctx, sensorId).Execute()

Get all sensor data by id



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
	openapiclient "github.com/green-ecolution/green-ecolution/backend/pkg/client"
)

func main() {
	sensorId := "sensorId_example" // string | Sensor ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.SensorAPI.GetAllSensorDataById(context.Background(), sensorId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `SensorAPI.GetAllSensorDataById``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllSensorDataById`: SensorDataList
	fmt.Fprintf(os.Stdout, "Response from `SensorAPI.GetAllSensorDataById`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**sensorId** | **string** | Sensor ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetAllSensorDataByIdRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**SensorDataList**](SensorDataList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetAllSensors

> SensorList GetAllSensors(ctx).Page(page).Limit(limit).Provider(provider).Execute()

Get all sensors



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
	openapiclient "github.com/green-ecolution/green-ecolution/backend/pkg/client"
)

func main() {
	page := int32(56) // int32 | Page (optional)
	limit := int32(56) // int32 | Limit (optional)
	provider := "provider_example" // string | Provider (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.SensorAPI.GetAllSensors(context.Background()).Page(page).Limit(limit).Provider(provider).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `SensorAPI.GetAllSensors``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllSensors`: SensorList
	fmt.Fprintf(os.Stdout, "Response from `SensorAPI.GetAllSensors`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiGetAllSensorsRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **page** | **int32** | Page | 
 **limit** | **int32** | Limit | 
 **provider** | **string** | Provider | 

### Return type

[**SensorList**](SensorList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetSensorById

> Sensor GetSensorById(ctx, sensorId).Execute()

Get sensor by ID



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
	openapiclient "github.com/green-ecolution/green-ecolution/backend/pkg/client"
)

func main() {
	sensorId := "sensorId_example" // string | Sensor ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.SensorAPI.GetSensorById(context.Background(), sensorId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `SensorAPI.GetSensorById``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetSensorById`: Sensor
	fmt.Fprintf(os.Stdout, "Response from `SensorAPI.GetSensorById`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**sensorId** | **string** | Sensor ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetSensorByIdRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**Sensor**](Sensor.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

