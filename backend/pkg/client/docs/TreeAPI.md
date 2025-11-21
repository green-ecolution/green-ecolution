# \TreeAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**CreateTree**](TreeAPI.md#CreateTree) | **Post** /v1/tree | Create tree
[**DeleteTree**](TreeAPI.md#DeleteTree) | **Delete** /v1/tree/{tree_id} | Delete tree
[**GetAllTrees**](TreeAPI.md#GetAllTrees) | **Get** /v1/tree | Get all trees
[**GetTrees**](TreeAPI.md#GetTrees) | **Get** /v1/tree/{tree_id} | Get tree by ID
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
	body := *openapiclient.NewTreeCreate("Description_example", float32(123), float32(123), "Number_example", int32(123), "Species_example") // TreeCreate | Tree to create

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
 **body** | [**TreeCreate**](TreeCreate.md) | Tree to create | 

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
	page := int32(56) // int32 | Page (optional)
	limit := int32(56) // int32 | Limit (optional)
	provider := "provider_example" // string | Provider (optional)
	wateringStatuses := []string{"Inner_example"} // []string | watering status (good, moderate, bad) (optional)
	plantingYears := []int32{int32(123)} // []int32 | planting_years (optional)
	hasCluster := true // bool | has cluster (optional)

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
 **page** | **int32** | Page | 
 **limit** | **int32** | Limit | 
 **provider** | **string** | Provider | 
 **wateringStatuses** | **[]string** | watering status (good, moderate, bad) | 
 **plantingYears** | **[]int32** | planting_years | 
 **hasCluster** | **bool** | has cluster | 

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


## GetTrees

> Tree GetTrees(ctx, treeId).Execute()

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
	resp, r, err := apiClient.TreeAPI.GetTrees(context.Background(), treeId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeAPI.GetTrees``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetTrees`: Tree
	fmt.Fprintf(os.Stdout, "Response from `TreeAPI.GetTrees`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**treeId** | **int32** | Tree ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetTreesRequest struct via the builder pattern


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
	body := *openapiclient.NewTreeUpdate("Description_example", float32(123), float32(123), "Number_example", int32(123), "Species_example") // TreeUpdate | Tree to update

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

 **body** | [**TreeUpdate**](TreeUpdate.md) | Tree to update | 

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

