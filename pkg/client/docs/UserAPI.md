# \UserAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GetAllUsers**](UserAPI.md#GetAllUsers) | **Get** /v1/user | Get all users
[**GetUsersByRole**](UserAPI.md#GetUsersByRole) | **Get** /v1/user/role/{role} | Get users by role
[**V1UserLoginGet**](UserAPI.md#V1UserLoginGet) | **Get** /v1/user/login | Request to login
[**V1UserLoginTokenPost**](UserAPI.md#V1UserLoginTokenPost) | **Post** /v1/user/login/token | Validate login code and request a access token
[**V1UserLogoutPost**](UserAPI.md#V1UserLogoutPost) | **Post** /v1/user/logout | Logout from the system
[**V1UserPost**](UserAPI.md#V1UserPost) | **Post** /v1/user | Register a new user
[**V1UserTokenRefreshPost**](UserAPI.md#V1UserTokenRefreshPost) | **Post** /v1/user/token/refresh | Refresh token



## GetAllUsers

> UserList GetAllUsers(ctx).UserIds(userIds).Execute()

Get all users



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
	userIds := "userIds_example" // string | User IDs (optional)

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.GetAllUsers(context.Background()).UserIds(userIds).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.GetAllUsers``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetAllUsers`: UserList
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.GetAllUsers`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiGetAllUsersRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **userIds** | **string** | User IDs | 

### Return type

[**UserList**](UserList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## GetUsersByRole

> UserList GetUsersByRole(ctx, role).Execute()

Get users by role



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
	role := "role_example" // string | Role

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.GetUsersByRole(context.Background(), role).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.GetUsersByRole``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `GetUsersByRole`: UserList
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.GetUsersByRole`: %v\n", resp)
}
```

### Path Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
**role** | **string** | Role | 

### Other Parameters

Other parameters are passed through a pointer to a apiGetUsersByRoleRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------


### Return type

[**UserList**](UserList.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## V1UserLoginGet

> LoginResponse V1UserLoginGet(ctx).RedirectUrl(redirectUrl).Execute()

Request to login

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
	redirectUrl := "redirectUrl_example" // string | Redirect URL

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.V1UserLoginGet(context.Background()).RedirectUrl(redirectUrl).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.V1UserLoginGet``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `V1UserLoginGet`: LoginResponse
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.V1UserLoginGet`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiV1UserLoginGetRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **redirectUrl** | **string** | Redirect URL | 

### Return type

[**LoginResponse**](LoginResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## V1UserLoginTokenPost

> ClientToken V1UserLoginTokenPost(ctx).RedirectUrl(redirectUrl).Body(body).Execute()

Validate login code and request a access token

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
	redirectUrl := "redirectUrl_example" // string | Redirect URL
	body := *openapiclient.NewLoginTokenRequest("Code_example") // LoginTokenRequest | Callback information

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.V1UserLoginTokenPost(context.Background()).RedirectUrl(redirectUrl).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.V1UserLoginTokenPost``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `V1UserLoginTokenPost`: ClientToken
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.V1UserLoginTokenPost`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiV1UserLoginTokenPostRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **redirectUrl** | **string** | Redirect URL | 
 **body** | [**LoginTokenRequest**](LoginTokenRequest.md) | Callback information | 

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


## V1UserLogoutPost

> string V1UserLogoutPost(ctx).Body(body).Execute()

Logout from the system

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
	body := *openapiclient.NewLogoutRequest("RefreshToken_example") // LogoutRequest | Logout information

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.V1UserLogoutPost(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.V1UserLogoutPost``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `V1UserLogoutPost`: string
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.V1UserLogoutPost`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiV1UserLogoutPostRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**LogoutRequest**](LogoutRequest.md) | Logout information | 

### Return type

**string**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: */*

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## V1UserPost

> User V1UserPost(ctx).User(user).Execute()

Register a new user



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
	user := *openapiclient.NewUserRegister("AvatarUrl_example", "Email_example", "EmployeeId_example", "FirstName_example", "LastName_example", "Password_example", "PhoneNumber_example", []string{"Roles_example"}, "Username_example") // UserRegister | User information

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.V1UserPost(context.Background()).User(user).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.V1UserPost``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `V1UserPost`: User
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.V1UserPost`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiV1UserPostRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user** | [**UserRegister**](UserRegister.md) | User information | 

### Return type

[**User**](User.md)

### Authorization

[Keycloak](../README.md#Keycloak)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## V1UserTokenRefreshPost

> ClientToken V1UserTokenRefreshPost(ctx).Body(body).Execute()

Refresh token



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
	body := *openapiclient.NewRefreshTokenRequest("RefreshToken_example") // RefreshTokenRequest | Refresh token information

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.V1UserTokenRefreshPost(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.V1UserTokenRefreshPost``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `V1UserTokenRefreshPost`: ClientToken
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.V1UserTokenRefreshPost`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiV1UserTokenRefreshPostRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**RefreshTokenRequest**](RefreshTokenRequest.md) | Refresh token information | 

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

