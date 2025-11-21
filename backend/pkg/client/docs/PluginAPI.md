# \PluginAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GetPluginInfo**](PluginAPI.md#GetPluginInfo) | **Get** /v1/plugin/{plugin_slug} | Get a plugin info
[**GetPluginsList**](PluginAPI.md#GetPluginsList) | **Get** /v1/plugin | Get a list of all registered plugins
[**PluginHeartbeat**](PluginAPI.md#PluginHeartbeat) | **Post** /v1/plugin/{plugin_slug}/heartbeat | Heartbeat for a plugin
[**RefreshPluginToken**](PluginAPI.md#RefreshPluginToken) | **Post** /v1/plugin/{plugin_slug}/token/refresh | Refresh plugin token
[**RegisterPlugin**](PluginAPI.md#RegisterPlugin) | **Post** /v1/plugin | Register a plugin
[**UnregisterPlugin**](PluginAPI.md#UnregisterPlugin) | **Post** /v1/plugin/{plugin_slug}/unregister | Unregister a plugin



## GetPluginInfo

> Plugin GetPluginInfo(ctx, pluginSlug).Execute()

Get a plugin info



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
	pluginSlug := "pluginSlug_example" // string | Slug of the plugin

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.PluginAPI.GetPluginInfo(context.Background(), pluginSlug).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `PluginAPI.GetPluginInfo``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetPluginInfo`: Plugin
	fmt.Fprintf(os.Stdout, "Response from `PluginAPI.GetPluginInfo`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**pluginSlug** | **string** | Slug of the plugin | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetPluginInfoRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**Plugin**](Plugin.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetPluginsList

> PluginListResponse GetPluginsList(ctx).Execute()

Get a list of all registered plugins



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
	resp, r, err := apiClient.PluginAPI.GetPluginsList(context.Background()).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `PluginAPI.GetPluginsList``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetPluginsList`: PluginListResponse
	fmt.Fprintf(os.Stdout, "Response from `PluginAPI.GetPluginsList`: %v\n", resp)
}
```

### Path Parameters

This endpoint does not need any parameter.

### Other Parameters

Other parameters are passed through a pointer to a apiGetPluginsListRequest struct via the builder pattern


### Return type

[**PluginListResponse**](PluginListResponse.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## PluginHeartbeat

> string PluginHeartbeat(ctx, pluginSlug).Execute()

Heartbeat for a plugin



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
	pluginSlug := "pluginSlug_example" // string | Name of the plugin specified by slug during registration

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.PluginAPI.PluginHeartbeat(context.Background(), pluginSlug).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `PluginAPI.PluginHeartbeat``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `PluginHeartbeat`: string
	fmt.Fprintf(os.Stdout, "Response from `PluginAPI.PluginHeartbeat`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**pluginSlug** | **string** | Name of the plugin specified by slug during registration | 

### Other Parameters

Other parameters are passed through a pointer to a apiPluginHeartbeatRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

**string**

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## RefreshPluginToken

> ClientToken RefreshPluginToken(ctx, pluginSlug).Body(body).Execute()

Refresh plugin token



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
	pluginSlug := "pluginSlug_example" // string | Slug of the plugin
	body := *openapiclient.NewPluginAuth("ClientId_example", "ClientSecret_example") // PluginAuth | Plugin authentication

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.PluginAPI.RefreshPluginToken(context.Background(), pluginSlug).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `PluginAPI.RefreshPluginToken``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `RefreshPluginToken`: ClientToken
	fmt.Fprintf(os.Stdout, "Response from `PluginAPI.RefreshPluginToken`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**pluginSlug** | **string** | Slug of the plugin | 

### Other Parameters

Other parameters are passed through a pointer to a apiRefreshPluginTokenRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------

 **body** | [**PluginAuth**](PluginAuth.md) | Plugin authentication | 

### Return type

[**ClientToken**](ClientToken.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## RegisterPlugin

> ClientToken RegisterPlugin(ctx).Body(body).Execute()

Register a plugin



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
	body := *openapiclient.NewPluginRegisterRequest(*openapiclient.NewPluginAuth("ClientId_example", "ClientSecret_example"), "Description_example", "Name_example", "Path_example", "Slug_example", "Version_example") // PluginRegisterRequest | Plugin registration request

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.PluginAPI.RegisterPlugin(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `PluginAPI.RegisterPlugin``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `RegisterPlugin`: ClientToken
	fmt.Fprintf(os.Stdout, "Response from `PluginAPI.RegisterPlugin`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiRegisterPluginRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**PluginRegisterRequest**](PluginRegisterRequest.md) | Plugin registration request | 

### Return type

[**ClientToken**](ClientToken.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## UnregisterPlugin

> UnregisterPlugin(ctx, pluginSlug).Execute()

Unregister a plugin



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
	pluginSlug := "pluginSlug_example" // string | Slug of the plugin

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	r, err := apiClient.PluginAPI.UnregisterPlugin(context.Background(), pluginSlug).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `PluginAPI.UnregisterPlugin``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**pluginSlug** | **string** | Slug of the plugin | 

### Other Parameters

Other parameters are passed through a pointer to a apiUnregisterPluginRequest struct via the builder pattern


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

