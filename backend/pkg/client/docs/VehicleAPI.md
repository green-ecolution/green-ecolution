# \VehicleAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ArchiveVehicle**](VehicleAPI.md#ArchiveVehicle) | **Post** /v1/vehicle/archive/{id} | Archive vehicle
[**CreateVehicle**](VehicleAPI.md#CreateVehicle) | **Post** /v1/vehicle | Create vehicle
[**DeleteVehicle**](VehicleAPI.md#DeleteVehicle) | **Delete** /v1/vehicle/{id} | Delete vehicle
[**GetAllVehicles**](VehicleAPI.md#GetAllVehicles) | **Get** /v1/vehicle | Get all vehicles
[**GetArchiveVehicle**](VehicleAPI.md#GetArchiveVehicle) | **Get** /v1/vehicle/archive | Get archived vehicle
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
	openapiclient "github.com/green-ecolution/backend/pkg/client"
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
	openapiclient "github.com/green-ecolution/backend/pkg/client"
)

func main() {
	body := *openapiclient.NewVehicleCreate("Description_example", openapiclient.DrivingLicense("B"), float32(123), float32(123), "Model_example", "NumberPlate_example", openapiclient.VehicleStatus("active"), openapiclient.VehicleType("transporter"), float32(123), float32(123), float32(123)) // VehicleCreate | Vehicle Create Request

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
 **body** | [**VehicleCreate**](VehicleCreate.md) | Vehicle Create Request | 

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
	openapiclient "github.com/green-ecolution/backend/pkg/client"
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
	openapiclient "github.com/green-ecolution/backend/pkg/client"
)

func main() {
	page := int32(56) // int32 | Page (optional)
	limit := int32(56) // int32 | Limit (optional)
	type_ := "type__example" // string | Vehicle Type (optional)
	provider := "provider_example" // string | Provider (optional)
	archived := true // bool | With archived vehicles (optional)

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
 **page** | **int32** | Page | 
 **limit** | **int32** | Limit | 
 **type_** | **string** | Vehicle Type | 
 **provider** | **string** | Provider | 
 **archived** | **bool** | With archived vehicles | 

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


## GetArchiveVehicle

> []Vehicle GetArchiveVehicle(ctx).Execute()

Get archived vehicle



### Example

```go
package main

import (
	"context"
	"fmt"
	"os"
	openapiclient "github.com/green-ecolution/backend/pkg/client"
)

func main() {

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.VehicleAPI.GetArchiveVehicle(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `VehicleAPI.GetArchiveVehicle``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetArchiveVehicle`: []Vehicle
	fmt.Fprintf(os.Stdout, "Response from `VehicleAPI.GetArchiveVehicle`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetArchiveVehicleRequest struct via the builder pattern


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
	openapiclient "github.com/green-ecolution/backend/pkg/client"
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
	openapiclient "github.com/green-ecolution/backend/pkg/client"
)

func main() {
	plate := "plate_example" // string | Vehicle plate number

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
**plate** | **string** | Vehicle plate number | 

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
	openapiclient "github.com/green-ecolution/backend/pkg/client"
)

func main() {
	id := "id_example" // string | Vehicle ID
	body := *openapiclient.NewVehicleUpdate("Description_example", openapiclient.DrivingLicense("B"), float32(123), float32(123), "Model_example", "NumberPlate_example", openapiclient.VehicleStatus("active"), openapiclient.VehicleType("transporter"), float32(123), float32(123), float32(123)) // VehicleUpdate | Vehicle Update Request

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
**id** | **string** | Vehicle ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiUpdateVehicleRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------

 **body** | [**VehicleUpdate**](VehicleUpdate.md) | Vehicle Update Request | 

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

