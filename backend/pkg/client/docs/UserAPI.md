# \UserAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**GetAllUsers**](UserAPI.md#GetAllUsers) | **Get** /v1/user | Get all users
[**GetUsersByRole**](UserAPI.md#GetUsersByRole) | **Get** /v1/user/role/{role} | Get users by role
[**Login**](UserAPI.md#Login) | **Get** /v1/user/login | Request to login
[**Logout**](UserAPI.md#Logout) | **Post** /v1/user/logout | Logout from the system
[**RefreshToken**](UserAPI.md#RefreshToken) | **Post** /v1/user/token/refresh | Refresh token
[**RegisterUser**](UserAPI.md#RegisterUser) | **Post** /v1/user | Register a new user
[**RequestToken**](UserAPI.md#RequestToken) | **Post** /v1/user/login/token | Request access token



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
	openapiclient "github.com/green-ecolution/green-ecolution/backend/pkg/client"
)

func main() {
	userIds := "userIds_example" // string | Comma-separated list of user IDs to filter (optional)

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
 **userIds** | **string** | Comma-separated list of user IDs to filter | 

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
	openapiclient "github.com/green-ecolution/green-ecolution/backend/pkg/client"
)

func main() {
	role := "role_example" // string | Role name to filter users by

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
**role** | **string** | Role name to filter users by | 

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


## Login

> LoginResponse Login(ctx).RedirectUrl(redirectUrl).Execute()

Request to login



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
	redirectUrl := "redirectUrl_example" // string | URL to redirect back after authentication

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.Login(context.Background()).RedirectUrl(redirectUrl).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.Login``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `Login`: LoginResponse
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.Login`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiLoginRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **redirectUrl** | **string** | URL to redirect back after authentication | 

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


## Logout

> string Logout(ctx).Body(body).Execute()

Logout from the system



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
	body := *openapiclient.NewLogoutRequest("RefreshToken_example") // LogoutRequest | Logout request with refresh token

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.Logout(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.Logout``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `Logout`: string
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.Logout`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiLogoutRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**LogoutRequest**](LogoutRequest.md) | Logout request with refresh token | 

### Return type

**string**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: */*

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)


## RefreshToken

> ClientToken RefreshToken(ctx).Body(body).Execute()

Refresh token



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
	body := *openapiclient.NewRefreshTokenRequest("RefreshToken_example") // RefreshTokenRequest | Refresh token to exchange

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.RefreshToken(context.Background()).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.RefreshToken``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `RefreshToken`: ClientToken
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.RefreshToken`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiRefreshTokenRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | [**RefreshTokenRequest**](RefreshTokenRequest.md) | Refresh token to exchange | 

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


## RegisterUser

> User RegisterUser(ctx).User(user).Execute()

Register a new user



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
	user := *openapiclient.NewUserRegister("Email_example", "FirstName_example", "LastName_example", "Password_example", []string{"Roles_example"}, "Username_example") // UserRegister | User registration data

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.RegisterUser(context.Background()).User(user).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.RegisterUser``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `RegisterUser`: User
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.RegisterUser`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiRegisterUserRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **user** | [**UserRegister**](UserRegister.md) | User registration data | 

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


## RequestToken

> ClientToken RequestToken(ctx).RedirectUrl(redirectUrl).Body(body).Execute()

Request access token



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
	redirectUrl := "redirectUrl_example" // string | Same redirect URL used in login request
	body := *openapiclient.NewLoginTokenRequest("Code_example") // LoginTokenRequest | Authorization code from OAuth2 callback

	configuration := openapiclient.NewConfiguration()
	apiClient := openapiclient.NewAPIClient(configuration)
	resp, r, err := apiClient.UserAPI.RequestToken(context.Background()).RedirectUrl(redirectUrl).Body(body).Execute()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error when calling `UserAPI.RequestToken``: %v\n", err)
		fmt.Fprintf(os.Stderr, "Full HTTP response: %v\n", r)
	}
	// response from `RequestToken`: ClientToken
	fmt.Fprintf(os.Stdout, "Response from `UserAPI.RequestToken`: %v\n", resp)
}
```

### Path Parameters



### Other Parameters

Other parameters are passed through a pointer to a apiRequestTokenRequest struct via the builder pattern


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **redirectUrl** | **string** | Same redirect URL used in login request | 
 **body** | [**LoginTokenRequest**](LoginTokenRequest.md) | Authorization code from OAuth2 callback | 

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

