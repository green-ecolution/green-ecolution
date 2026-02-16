# \TreeAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**CreateTree**](TreeAPI.md#CreateTree) | **Post** /v1/tree | Create tree
[**DeleteTree**](TreeAPI.md#DeleteTree) | **Delete** /v1/tree/{tree_id} | Delete tree
[**GetAllTrees**](TreeAPI.md#GetAllTrees) | **Get** /v1/tree | Get all trees
[**GetPlantingYears**](TreeAPI.md#GetPlantingYears) | **Get** /v1/tree/planting-years | Get distinct planting years
[**GetTreeById**](TreeAPI.md#GetTreeById) | **Get** /v1/tree/{tree_id} | Get tree by ID
[**UpdateTree**](TreeAPI.md#UpdateTree) | **Put** /v1/tree/{tree_id} | Update tree



## CreateTree

> Tree CreateTree(ctx).Body(body).Execute()

Create tree



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
	body := *openapiclient.NewTreeCreate(map[string]interface{}{"key": interface{}(123)}, "Description_example", float32(123), float32(123), "Number_example", int32(123), "Provider_example", "SensorId_example", "Species_example", int32(123)) // TreeCreate | Tree data to create

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeAPI.CreateTree(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeAPI.CreateTree``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `CreateTree`: Tree
	fmt.Fprintf(os.Stdout, "Response from `TreeAPI.CreateTree`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiCreateTreeRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**TreeCreate**](TreeCreate.md) | Tree data to create | 

### Return type

[**Tree**](Tree.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## DeleteTree

> DeleteTree(ctx, treeId).Execute()

Delete tree



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
	treeId := int32(56) // int32 | Tree ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	r, err := apiClient.TreeAPI.DeleteTree(context.Background(), treeId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeAPI.DeleteTree``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**treeId** | **int32** | Tree ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiDeleteTreeRequest struct via the builder pattern


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


## GetAllTrees

> TreeList GetAllTrees(ctx).Page(page).Limit(limit).Provider(provider).WateringStatuses(wateringStatuses).PlantingYears(plantingYears).HasCluster(hasCluster).Execute()

Get all trees



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
	wateringStatuses := []string{"Inner_example"} // []string | Filter by watering status (good, moderate, bad) (optional)
	plantingYears := []int32{int32(123)} // []int32 | Filter by planting years (optional)
	hasCluster := true // bool | Filter trees that belong to a cluster (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeAPI.GetAllTrees(context.Background()).Page(page).Limit(limit).Provider(provider).WateringStatuses(wateringStatuses).PlantingYears(plantingYears).HasCluster(hasCluster).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeAPI.GetAllTrees``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllTrees`: TreeList
	fmt.Fprintf(os.Stdout, "Response from `TreeAPI.GetAllTrees`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiGetAllTreesRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **page** | **int32** | Page number for pagination | 
 **limit** | **int32** | Number of items per page | 
 **provider** | **string** | Filter by data provider | 
 **wateringStatuses** | **[]string** | Filter by watering status (good, moderate, bad) | 
 **plantingYears** | **[]int32** | Filter by planting years | 
 **hasCluster** | **bool** | Filter trees that belong to a cluster | 

### Return type

[**TreeList**](TreeList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetPlantingYears

> []int32 GetPlantingYears(ctx).Execute()

Get distinct planting years



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
	resp, r, err := apiClient.TreeAPI.GetPlantingYears(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeAPI.GetPlantingYears``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetPlantingYears`: []int32
	fmt.Fprintf(os.Stdout, "Response from `TreeAPI.GetPlantingYears`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetPlantingYearsRequest struct via the builder pattern


### Return type

**[]int32**

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetTreeById

> Tree GetTreeById(ctx, treeId).Execute()

Get tree by ID



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
	treeId := int32(56) // int32 | Tree ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeAPI.GetTreeById(context.Background(), treeId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeAPI.GetTreeById``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetTreeById`: Tree
	fmt.Fprintf(os.Stdout, "Response from `TreeAPI.GetTreeById`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**treeId** | **int32** | Tree ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetTreeByIdRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**Tree**](Tree.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## UpdateTree

> Tree UpdateTree(ctx, treeId).Body(body).Execute()

Update tree



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
	treeId := int32(56) // int32 | Tree ID
	body := *openapiclient.NewTreeUpdate(map[string]interface{}{"key": interface{}(123)}, "Description_example", float32(123), float32(123), "Number_example", int32(123), "Provider_example", "SensorId_example", "Species_example", int32(123)) // TreeUpdate | Tree data to update

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeAPI.UpdateTree(context.Background(), treeId).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeAPI.UpdateTree``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `UpdateTree`: Tree
	fmt.Fprintf(os.Stdout, "Response from `TreeAPI.UpdateTree`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**treeId** | **int32** | Tree ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiUpdateTreeRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------

 **body** | [**TreeUpdate**](TreeUpdate.md) | Tree data to update | 

### Return type

[**Tree**](Tree.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

