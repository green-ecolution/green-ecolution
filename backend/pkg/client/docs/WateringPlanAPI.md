# \WateringPlanAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**CreatePreviewRoute**](WateringPlanAPI.md#CreatePreviewRoute) | **Post** /v1/watering-plan/route/preview | Generate preview route
[**CreateWateringPlan**](WateringPlanAPI.md#CreateWateringPlan) | **Post** /v1/watering-plan | Create watering plan
[**DeleteWateringPlan**](WateringPlanAPI.md#DeleteWateringPlan) | **Delete** /v1/watering-plan/{id} | Delete watering plan
[**GetAllWateringPlans**](WateringPlanAPI.md#GetAllWateringPlans) | **Get** /v1/watering-plan | Get all watering plans
[**GetGpxFile**](WateringPlanAPI.md#GetGpxFile) | **Get** /v1/watering-plan/route/gpx/{gpx_name} | Download GPX file
[**GetWateringPlanById**](WateringPlanAPI.md#GetWateringPlanById) | **Get** /v1/watering-plan/{id} | Get watering plan by ID
[**UpdateWateringPlan**](WateringPlanAPI.md#UpdateWateringPlan) | **Put** /v1/watering-plan/{id} | Update watering plan



## CreatePreviewRoute

> GeoJson CreatePreviewRoute(ctx).Body(body).Execute()

Generate preview route



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
	body := *openapiclient.NewRouteRequest([]int32{int32(123)}, int32(123)) // RouteRequest | Route preview request with vehicles and clusters

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.WateringPlanAPI.CreatePreviewRoute(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `WateringPlanAPI.CreatePreviewRoute``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `CreatePreviewRoute`: GeoJson
	fmt.Fprintf(os.Stdout, "Response from `WateringPlanAPI.CreatePreviewRoute`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiCreatePreviewRouteRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**RouteRequest**](RouteRequest.md) | Route preview request with vehicles and clusters | 

### Return type

[**GeoJson**](GeoJson.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## CreateWateringPlan

> WateringPlan CreateWateringPlan(ctx).Body(body).Execute()

Create watering plan



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
	body := *openapiclient.NewWateringPlanCreate("Date_example", "Description_example", int32(123), []int32{int32(123)}, []string{"UserIds_example"}) // WateringPlanCreate | Watering plan data to create

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.WateringPlanAPI.CreateWateringPlan(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `WateringPlanAPI.CreateWateringPlan``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `CreateWateringPlan`: WateringPlan
	fmt.Fprintf(os.Stdout, "Response from `WateringPlanAPI.CreateWateringPlan`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiCreateWateringPlanRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**WateringPlanCreate**](WateringPlanCreate.md) | Watering plan data to create | 

### Return type

[**WateringPlan**](WateringPlan.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## DeleteWateringPlan

> DeleteWateringPlan(ctx, id).Execute()

Delete watering plan



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
	id := int32(56) // int32 | Watering Plan ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	r, err := apiClient.WateringPlanAPI.DeleteWateringPlan(context.Background(), id).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `WateringPlanAPI.DeleteWateringPlan``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Watering Plan ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiDeleteWateringPlanRequest struct via the builder pattern


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


## GetAllWateringPlans

> WateringPlanList GetAllWateringPlans(ctx).Page(page).Limit(limit).Provider(provider).Execute()

Get all watering plans



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
	provider := "provider_example" // string | Filter by data provider (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.WateringPlanAPI.GetAllWateringPlans(context.Background()).Page(page).Limit(limit).Provider(provider).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `WateringPlanAPI.GetAllWateringPlans``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllWateringPlans`: WateringPlanList
	fmt.Fprintf(os.Stdout, "Response from `WateringPlanAPI.GetAllWateringPlans`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiGetAllWateringPlansRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **page** | **int32** | Page number for pagination | 
 **limit** | **int32** | Number of items per page | 
 **provider** | **string** | Filter by data provider | 

### Return type

[**WateringPlanList**](WateringPlanList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetGpxFile

> *os.File GetGpxFile(ctx, gpxName).Execute()

Download GPX file



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
	gpxName := "gpxName_example" // string | GPX file name

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.WateringPlanAPI.GetGpxFile(context.Background(), gpxName).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `WateringPlanAPI.GetGpxFile``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetGpxFile`: *os.File
	fmt.Fprintf(os.Stdout, "Response from `WateringPlanAPI.GetGpxFile`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**gpxName** | **string** | GPX file name | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetGpxFileRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[***os.File**](*os.File.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/gpx+xml

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetWateringPlanById

> WateringPlan GetWateringPlanById(ctx, id).Execute()

Get watering plan by ID



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
	id := int32(56) // int32 | Watering Plan ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.WateringPlanAPI.GetWateringPlanById(context.Background(), id).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `WateringPlanAPI.GetWateringPlanById``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetWateringPlanById`: WateringPlan
	fmt.Fprintf(os.Stdout, "Response from `WateringPlanAPI.GetWateringPlanById`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Watering Plan ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetWateringPlanByIdRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**WateringPlan**](WateringPlan.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## UpdateWateringPlan

> WateringPlan UpdateWateringPlan(ctx, id).Body(body).Execute()

Update watering plan



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
	id := int32(56) // int32 | Watering Plan ID
	body := *openapiclient.NewWateringPlanUpdate("CancellationNote_example", "Date_example", "Description_example", openapiclient.WateringPlanStatus("planned"), int32(123), []int32{int32(123)}, []string{"UserIds_example"}) // WateringPlanUpdate | Watering plan data to update

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.WateringPlanAPI.UpdateWateringPlan(context.Background(), id).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `WateringPlanAPI.UpdateWateringPlan``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `UpdateWateringPlan`: WateringPlan
	fmt.Fprintf(os.Stdout, "Response from `WateringPlanAPI.UpdateWateringPlan`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Watering Plan ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiUpdateWateringPlanRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------

 **body** | [**WateringPlanUpdate**](WateringPlanUpdate.md) | Watering plan data to update | 

### Return type

[**WateringPlan**](WateringPlan.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

