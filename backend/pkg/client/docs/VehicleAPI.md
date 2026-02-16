# \VehicleAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ArchiveVehicle**](VehicleAPI.md#ArchiveVehicle) | **Post** /v1/vehicle/archive/{id} | Archive vehicle
[**CreateVehicle**](VehicleAPI.md#CreateVehicle) | **Post** /v1/vehicle | Create vehicle
[**DeleteVehicle**](VehicleAPI.md#DeleteVehicle) | **Delete** /v1/vehicle/{id} | Delete vehicle
[**GetAllVehicles**](VehicleAPI.md#GetAllVehicles) | **Get** /v1/vehicle | Get all vehicles
[**GetArchivedVehicles**](VehicleAPI.md#GetArchivedVehicles) | **Get** /v1/vehicle/archive | Get archived vehicles
[**GetVehicleById**](VehicleAPI.md#GetVehicleById) | **Get** /v1/vehicle/{id} | Get vehicle by ID
[**GetVehicleByPlate**](VehicleAPI.md#GetVehicleByPlate) | **Get** /v1/vehicle/plate/{plate} | Get vehicle by plate
[**UpdateVehicle**](VehicleAPI.md#UpdateVehicle) | **Put** /v1/vehicle/{id} | Update vehicle



## ArchiveVehicle

> ArchiveVehicle(ctx, id).Execute()

Archive vehicle



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
	id := int32(56) // int32 | Vehicle ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	r, err := apiClient.VehicleAPI.ArchiveVehicle(context.Background(), id).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.ArchiveVehicle``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Vehicle ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiArchiveVehicleRequest struct via the builder pattern


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


## CreateVehicle

> Vehicle CreateVehicle(ctx).Body(body).Execute()

Create vehicle



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
	body := *openapiclient.NewVehicleCreate(map[string]interface{}{"key": interface{}(123)}, "Description_example", openapiclient.DrivingLicense("B"), float32(123), float32(123), "Model_example", "NumberPlate_example", "Provider_example", openapiclient.VehicleStatus("active"), openapiclient.VehicleType("transporter"), float32(123), float32(123), float32(123)) // VehicleCreate | Vehicle data to create

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.VehicleAPI.CreateVehicle(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.CreateVehicle``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `CreateVehicle`: Vehicle
	fmt.Fprintf(os.Stdout, "Response from `VehicleAPI.CreateVehicle`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiCreateVehicleRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**VehicleCreate**](VehicleCreate.md) | Vehicle data to create | 

### Return type

[**Vehicle**](Vehicle.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## DeleteVehicle

> DeleteVehicle(ctx, id).Execute()

Delete vehicle



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
	id := int32(56) // int32 | Vehicle ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	r, err := apiClient.VehicleAPI.DeleteVehicle(context.Background(), id).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.DeleteVehicle``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Vehicle ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiDeleteVehicleRequest struct via the builder pattern


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


## GetAllVehicles

> VehicleList GetAllVehicles(ctx).Page(page).Limit(limit).Type_(type_).Provider(provider).Archived(archived).Execute()

Get all vehicles



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
	page := int32(56) // int32 | Page number for pagination (optional)
	limit := int32(56) // int32 | Number of items per page (optional)
	type_ := "type__example" // string | Filter by vehicle type (optional)
	provider := "provider_example" // string | Filter by data provider (optional)
	archived := true // bool | Include archived vehicles (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.VehicleAPI.GetAllVehicles(context.Background()).Page(page).Limit(limit).Type_(type_).Provider(provider).Archived(archived).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.GetAllVehicles``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllVehicles`: VehicleList
	fmt.Fprintf(os.Stdout, "Response from `VehicleAPI.GetAllVehicles`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiGetAllVehiclesRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **page** | **int32** | Page number for pagination | 
 **limit** | **int32** | Number of items per page | 
 **type_** | **string** | Filter by vehicle type | 
 **provider** | **string** | Filter by data provider | 
 **archived** | **bool** | Include archived vehicles | 

### Return type

[**VehicleList**](VehicleList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetArchivedVehicles

> []Vehicle GetArchivedVehicles(ctx).Execute()

Get archived vehicles



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

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.VehicleAPI.GetArchivedVehicles(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.GetArchivedVehicles``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetArchivedVehicles`: []Vehicle
	fmt.Fprintf(os.Stdout, "Response from `VehicleAPI.GetArchivedVehicles`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetArchivedVehiclesRequest struct via the builder pattern


### Return type

[**[]Vehicle**](Vehicle.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetVehicleById

> Vehicle GetVehicleById(ctx, id).Execute()

Get vehicle by ID



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
	id := int32(56) // int32 | Vehicle ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.VehicleAPI.GetVehicleById(context.Background(), id).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.GetVehicleById``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetVehicleById`: Vehicle
	fmt.Fprintf(os.Stdout, "Response from `VehicleAPI.GetVehicleById`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Vehicle ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetVehicleByIdRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**Vehicle**](Vehicle.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetVehicleByPlate

> Vehicle GetVehicleByPlate(ctx, plate).Execute()

Get vehicle by plate



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
	plate := "plate_example" // string | Vehicle license plate number

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.VehicleAPI.GetVehicleByPlate(context.Background(), plate).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.GetVehicleByPlate``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetVehicleByPlate`: Vehicle
	fmt.Fprintf(os.Stdout, "Response from `VehicleAPI.GetVehicleByPlate`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**plate** | **string** | Vehicle license plate number | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetVehicleByPlateRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**Vehicle**](Vehicle.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## UpdateVehicle

> Vehicle UpdateVehicle(ctx, id).Body(body).Execute()

Update vehicle



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
	id := int32(56) // int32 | Vehicle ID
	body := *openapiclient.NewVehicleUpdate(map[string]interface{}{"key": interface{}(123)}, "Description_example", openapiclient.DrivingLicense("B"), float32(123), float32(123), "Model_example", "NumberPlate_example", "Provider_example", openapiclient.VehicleStatus("active"), openapiclient.VehicleType("transporter"), float32(123), float32(123), float32(123)) // VehicleUpdate | Vehicle data to update

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.VehicleAPI.UpdateVehicle(context.Background(), id).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.UpdateVehicle``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `UpdateVehicle`: Vehicle
	fmt.Fprintf(os.Stdout, "Response from `VehicleAPI.UpdateVehicle`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Vehicle ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiUpdateVehicleRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------

 **body** | [**VehicleUpdate**](VehicleUpdate.md) | Vehicle data to update | 

### Return type

[**Vehicle**](Vehicle.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

