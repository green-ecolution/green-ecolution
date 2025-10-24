# \TreeSensorAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GetTreeBySensorId**](TreeSensorAPI.md#GetTreeBySensorId) | **Get** /v1/tree/sensor/{sensor_id} | Get tree by sensor ID



## GetTreeBySensorId

> Tree GetTreeBySensorId(ctx, sensorId).Execute()

Get tree by sensor ID



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
	sensorId := "sensorId_example" // string | Sensor ID

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.TreeSensorAPI.GetTreeBySensorId(context.Background(), sensorId).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `TreeSensorAPI.GetTreeBySensorId``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetTreeBySensorId`: Tree
	fmt.Fprintf(os.Stdout, "Response from `TreeSensorAPI.GetTreeBySensorId`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**sensorId** | **string** | Sensor ID | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetTreeBySensorIdRequest struct via the builder pattern


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

