# \RegionAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**V1RegionGet**](RegionAPI.md#V1RegionGet) | **Get** /v1/region | Get all regions
[**V1RegionIdGet**](RegionAPI.md#V1RegionIdGet) | **Get** /v1/region/{id} | Get a region by ID



## V1RegionGet

> RegionList V1RegionGet(ctx).Page(page).Limit(limit).Execute()

Get all regions



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

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.RegionAPI.V1RegionGet(context.Background()).Page(page).Limit(limit).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `RegionAPI.V1RegionGet``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `V1RegionGet`: RegionList
	fmt.Fprintf(os.Stdout, "Response from `RegionAPI.V1RegionGet`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiV1RegionGetRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **page** | **int32** | Page | 
 **limit** | **int32** | Limit | 

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


## V1RegionIdGet

> Region V1RegionIdGet(ctx, id).Execute()

Get a region by ID



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
	id := "id_example" // string | Region ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.RegionAPI.V1RegionIdGet(context.Background(), id).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `RegionAPI.V1RegionIdGet``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `V1RegionIdGet`: Region
	fmt.Fprintf(os.Stdout, "Response from `RegionAPI.V1RegionIdGet`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**id** | **string** | Region ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiV1RegionIdGetRequest struct via the builder pattern


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

