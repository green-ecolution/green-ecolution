# \InfoAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GetAppInfo**](InfoAPI.md#GetAppInfo) | **Get** /v1/info | Get application info
[**GetDataStatistics**](InfoAPI.md#GetDataStatistics) | **Get** /v1/info/statistics | Get data statistics
[**GetMapInfo**](InfoAPI.md#GetMapInfo) | **Get** /v1/info/map | Get map configuration
[**GetServerInfo**](InfoAPI.md#GetServerInfo) | **Get** /v1/info/server | Get server information
[**GetServicesStatus**](InfoAPI.md#GetServicesStatus) | **Get** /v1/info/services | Get services status



## GetAppInfo

> AppInfo GetAppInfo(ctx).Execute()

Get application info



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
	resp, r, err := apiClient.InfoAPI.GetAppInfo(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `InfoAPI.GetAppInfo``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAppInfo`: AppInfo
	fmt.Fprintf(os.Stdout, "Response from `InfoAPI.GetAppInfo`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetAppInfoRequest struct via the builder pattern


### Return type

[**AppInfo**](AppInfo.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetDataStatistics

> DataStatistics GetDataStatistics(ctx).Execute()

Get data statistics



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
	resp, r, err := apiClient.InfoAPI.GetDataStatistics(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `InfoAPI.GetDataStatistics``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetDataStatistics`: DataStatistics
	fmt.Fprintf(os.Stdout, "Response from `InfoAPI.GetDataStatistics`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetDataStatisticsRequest struct via the builder pattern


### Return type

[**DataStatistics**](DataStatistics.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetMapInfo

> MapInfo GetMapInfo(ctx).Execute()

Get map configuration



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
	resp, r, err := apiClient.InfoAPI.GetMapInfo(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `InfoAPI.GetMapInfo``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetMapInfo`: MapInfo
	fmt.Fprintf(os.Stdout, "Response from `InfoAPI.GetMapInfo`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetMapInfoRequest struct via the builder pattern


### Return type

[**MapInfo**](MapInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetServerInfo

> ServerInfo GetServerInfo(ctx).Execute()

Get server information



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
	resp, r, err := apiClient.InfoAPI.GetServerInfo(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `InfoAPI.GetServerInfo``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetServerInfo`: ServerInfo
	fmt.Fprintf(os.Stdout, "Response from `InfoAPI.GetServerInfo`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetServerInfoRequest struct via the builder pattern


### Return type

[**ServerInfo**](ServerInfo.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetServicesStatus

> ServicesInfo GetServicesStatus(ctx).Execute()

Get services status



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
	resp, r, err := apiClient.InfoAPI.GetServicesStatus(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `InfoAPI.GetServicesStatus``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetServicesStatus`: ServicesInfo
	fmt.Fprintf(os.Stdout, "Response from `InfoAPI.GetServicesStatus`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetServicesStatusRequest struct via the builder pattern


### Return type

[**ServicesInfo**](ServicesInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)

