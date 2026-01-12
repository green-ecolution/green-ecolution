# \RegionAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GetAllRegions**](RegionAPI.md#GetAllRegions) | **Get** /v1/region | Get all regions
[**GetRegionById**](RegionAPI.md#GetRegionById) | **Get** /v1/region/{id} | Get a region by ID



## GetAllRegions

> RegionList GetAllRegions(ctx).Page(page).Limit(limit).Execute()

Get all regions



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

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.RegionAPI.GetAllRegions(context.Background()).Page(page).Limit(limit).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `RegionAPI.GetAllRegions``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllRegions`: RegionList
	fmt.Fprintf(os.Stdout, "Response from `RegionAPI.GetAllRegions`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiGetAllRegionsRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **page** | **int32** | Page number for pagination | 
 **limit** | **int32** | Number of items per page | 

### Return type

[**RegionList**](RegionList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetRegionById

> Region GetRegionById(ctx, id).Execute()

Get a region by ID



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
	id := int32(56) // int32 | Region ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.RegionAPI.GetRegionById(context.Background(), id).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `RegionAPI.GetRegionById``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetRegionById`: Region
	fmt.Fprintf(os.Stdout, "Response from `RegionAPI.GetRegionById`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **int32** | Region ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetRegionByIdRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**Region**](Region.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

