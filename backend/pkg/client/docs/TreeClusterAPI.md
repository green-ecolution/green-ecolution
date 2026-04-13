# \TreeClusterAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**CreateTreeCluster**](TreeClusterAPI.md#CreateTreeCluster) | **Post** /v1/cluster | Create tree cluster
[**DeleteTreeCluster**](TreeClusterAPI.md#DeleteTreeCluster) | **Delete** /v1/cluster/{cluster_id} | Delete tree cluster
[**GetAllTreeClusters**](TreeClusterAPI.md#GetAllTreeClusters) | **Get** /v1/cluster | Get all tree clusters
[**GetTreeClusterById**](TreeClusterAPI.md#GetTreeClusterById) | **Get** /v1/cluster/{cluster_id} | Get tree cluster by ID
[**UpdateTreeCluster**](TreeClusterAPI.md#UpdateTreeCluster) | **Put** /v1/cluster/{cluster_id} | Update tree cluster



## CreateTreeCluster

> TreeCluster CreateTreeCluster(ctx).Body(body).Execute()

Create tree cluster



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
	body := *openapiclient.NewTreeClusterCreate("Address_example", "Description_example", "Name_example", openapiclient.SoilCondition("schluffig"), []int32{int32(123)}) // TreeClusterCreate | Tree cluster data to create

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeClusterAPI.CreateTreeCluster(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeClusterAPI.CreateTreeCluster``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `CreateTreeCluster`: TreeCluster
	fmt.Fprintf(os.Stdout, "Response from `TreeClusterAPI.CreateTreeCluster`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiCreateTreeClusterRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**TreeClusterCreate**](TreeClusterCreate.md) | Tree cluster data to create | 

### Return type

[**TreeCluster**](TreeCluster.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## DeleteTreeCluster

> DeleteTreeCluster(ctx, clusterId).Execute()

Delete tree cluster



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
	clusterId := int32(56) // int32 | Tree Cluster ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	r, err := apiClient.TreeClusterAPI.DeleteTreeCluster(context.Background(), clusterId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeClusterAPI.DeleteTreeCluster``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**clusterId** | **int32** | Tree Cluster ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiDeleteTreeClusterRequest struct via the builder pattern


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


## GetAllTreeClusters

> TreeClusterList GetAllTreeClusters(ctx).Page(page).Limit(limit).WateringStatuses(wateringStatuses).Regions(regions).Provider(provider).Execute()

Get all tree clusters



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
	wateringStatuses := []string{"Inner_example"} // []string | Filter by watering statuses (good, moderate, bad) (optional)
	regions := []string{"Inner_example"} // []string | Filter by region names (optional)
	provider := "provider_example" // string | Filter by data provider (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeClusterAPI.GetAllTreeClusters(context.Background()).Page(page).Limit(limit).WateringStatuses(wateringStatuses).Regions(regions).Provider(provider).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeClusterAPI.GetAllTreeClusters``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllTreeClusters`: TreeClusterList
	fmt.Fprintf(os.Stdout, "Response from `TreeClusterAPI.GetAllTreeClusters`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiGetAllTreeClustersRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **page** | **int32** | Page number for pagination | 
 **limit** | **int32** | Number of items per page | 
 **wateringStatuses** | **[]string** | Filter by watering statuses (good, moderate, bad) | 
 **regions** | **[]string** | Filter by region names | 
 **provider** | **string** | Filter by data provider | 

### Return type

[**TreeClusterList**](TreeClusterList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetTreeClusterById

> TreeCluster GetTreeClusterById(ctx, clusterId).Execute()

Get tree cluster by ID



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
	clusterId := int32(56) // int32 | Tree Cluster ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeClusterAPI.GetTreeClusterById(context.Background(), clusterId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeClusterAPI.GetTreeClusterById``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetTreeClusterById`: TreeCluster
	fmt.Fprintf(os.Stdout, "Response from `TreeClusterAPI.GetTreeClusterById`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**clusterId** | **int32** | Tree Cluster ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetTreeClusterByIdRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**TreeCluster**](TreeCluster.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## UpdateTreeCluster

> TreeCluster UpdateTreeCluster(ctx, clusterId).Body(body).Execute()

Update tree cluster



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
	clusterId := int32(56) // int32 | Tree Cluster ID
	body := *openapiclient.NewTreeClusterUpdate("Address_example", "Description_example", "Name_example", openapiclient.SoilCondition("schluffig"), []int32{int32(123)}) // TreeClusterUpdate | Tree cluster data to update

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeClusterAPI.UpdateTreeCluster(context.Background(), clusterId).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeClusterAPI.UpdateTreeCluster``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `UpdateTreeCluster`: TreeCluster
	fmt.Fprintf(os.Stdout, "Response from `TreeClusterAPI.UpdateTreeCluster`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**clusterId** | **int32** | Tree Cluster ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiUpdateTreeClusterRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------

 **body** | [**TreeClusterUpdate**](TreeClusterUpdate.md) | Tree cluster data to update | 

### Return type

[**TreeCluster**](TreeCluster.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

