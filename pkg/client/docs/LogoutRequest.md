# LogoutRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**RefreshToken** | **string** |  | 

## Methods

### NewLogoutRequest

`func NewLogoutRequest(refreshToken string, ) *LogoutRequest`

NewLogoutRequest instantiates a new LogoutRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewLogoutRequestWithDefaults

`func NewLogoutRequestWithDefaults() *LogoutRequest`

NewLogoutRequestWithDefaults instantiates a new LogoutRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetRefreshToken

`func (o *LogoutRequest) GetRefreshToken() string`

GetRefreshToken returns the RefreshToken field if non-nil, zero value otherwise.

### GetRefreshTokenOk

`func (o *LogoutRequest) GetRefreshTokenOk() (*string, bool)`

GetRefreshTokenOk returns a tuple with the RefreshToken field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRefreshToken

`func (o *LogoutRequest) SetRefreshToken(v string)`

SetRefreshToken sets RefreshToken field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


