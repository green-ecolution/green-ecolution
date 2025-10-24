# ClientToken

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AccessToken** | **string** |  | 
**ExpiresIn** | **int32** |  | 
**Expiry** | **string** |  | 
**IdToken** | **string** |  | 
**NotBeforePolicy** | **int32** |  | 
**RefreshExpiresIn** | **int32** |  | 
**RefreshToken** | **string** |  | 
**Scope** | **string** |  | 
**SessionState** | **string** |  | 
**TokenType** | **string** |  | 

## Methods

### NewClientToken

`func NewClientToken(accessToken string, expiresIn int32, expiry string, idToken string, notBeforePolicy int32, refreshExpiresIn int32, refreshToken string, scope string, sessionState string, tokenType string, ) *ClientToken`

NewClientToken instantiates a new ClientToken object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewClientTokenWithDefaults

`func NewClientTokenWithDefaults() *ClientToken`

NewClientTokenWithDefaults instantiates a new ClientToken object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAccessToken

`func (o *ClientToken) GetAccessToken() string`

GetAccessToken returns the AccessToken field if non-nil, zero value otherwise.

### GetAccessTokenOk

`func (o *ClientToken) GetAccessTokenOk() (*string, bool)`

GetAccessTokenOk returns a tuple with the AccessToken field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAccessToken

`func (o *ClientToken) SetAccessToken(v string)`

SetAccessToken sets AccessToken field to given value.


### GetExpiresIn

`func (o *ClientToken) GetExpiresIn() int32`

GetExpiresIn returns the ExpiresIn field if non-nil, zero value otherwise.

### GetExpiresInOk

`func (o *ClientToken) GetExpiresInOk() (*int32, bool)`

GetExpiresInOk returns a tuple with the ExpiresIn field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExpiresIn

`func (o *ClientToken) SetExpiresIn(v int32)`

SetExpiresIn sets ExpiresIn field to given value.


### GetExpiry

`func (o *ClientToken) GetExpiry() string`

GetExpiry returns the Expiry field if non-nil, zero value otherwise.

### GetExpiryOk

`func (o *ClientToken) GetExpiryOk() (*string, bool)`

GetExpiryOk returns a tuple with the Expiry field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExpiry

`func (o *ClientToken) SetExpiry(v string)`

SetExpiry sets Expiry field to given value.


### GetIdToken

`func (o *ClientToken) GetIdToken() string`

GetIdToken returns the IdToken field if non-nil, zero value otherwise.

### GetIdTokenOk

`func (o *ClientToken) GetIdTokenOk() (*string, bool)`

GetIdTokenOk returns a tuple with the IdToken field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetIdToken

`func (o *ClientToken) SetIdToken(v string)`

SetIdToken sets IdToken field to given value.


### GetNotBeforePolicy

`func (o *ClientToken) GetNotBeforePolicy() int32`

GetNotBeforePolicy returns the NotBeforePolicy field if non-nil, zero value otherwise.

### GetNotBeforePolicyOk

`func (o *ClientToken) GetNotBeforePolicyOk() (*int32, bool)`

GetNotBeforePolicyOk returns a tuple with the NotBeforePolicy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNotBeforePolicy

`func (o *ClientToken) SetNotBeforePolicy(v int32)`

SetNotBeforePolicy sets NotBeforePolicy field to given value.


### GetRefreshExpiresIn

`func (o *ClientToken) GetRefreshExpiresIn() int32`

GetRefreshExpiresIn returns the RefreshExpiresIn field if non-nil, zero value otherwise.

### GetRefreshExpiresInOk

`func (o *ClientToken) GetRefreshExpiresInOk() (*int32, bool)`

GetRefreshExpiresInOk returns a tuple with the RefreshExpiresIn field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRefreshExpiresIn

`func (o *ClientToken) SetRefreshExpiresIn(v int32)`

SetRefreshExpiresIn sets RefreshExpiresIn field to given value.


### GetRefreshToken

`func (o *ClientToken) GetRefreshToken() string`

GetRefreshToken returns the RefreshToken field if non-nil, zero value otherwise.

### GetRefreshTokenOk

`func (o *ClientToken) GetRefreshTokenOk() (*string, bool)`

GetRefreshTokenOk returns a tuple with the RefreshToken field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRefreshToken

`func (o *ClientToken) SetRefreshToken(v string)`

SetRefreshToken sets RefreshToken field to given value.


### GetScope

`func (o *ClientToken) GetScope() string`

GetScope returns the Scope field if non-nil, zero value otherwise.

### GetScopeOk

`func (o *ClientToken) GetScopeOk() (*string, bool)`

GetScopeOk returns a tuple with the Scope field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScope

`func (o *ClientToken) SetScope(v string)`

SetScope sets Scope field to given value.


### GetSessionState

`func (o *ClientToken) GetSessionState() string`

GetSessionState returns the SessionState field if non-nil, zero value otherwise.

### GetSessionStateOk

`func (o *ClientToken) GetSessionStateOk() (*string, bool)`

GetSessionStateOk returns a tuple with the SessionState field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSessionState

`func (o *ClientToken) SetSessionState(v string)`

SetSessionState sets SessionState field to given value.


### GetTokenType

`func (o *ClientToken) GetTokenType() string`

GetTokenType returns the TokenType field if non-nil, zero value otherwise.

### GetTokenTypeOk

`func (o *ClientToken) GetTokenTypeOk() (*string, bool)`

GetTokenTypeOk returns a tuple with the TokenType field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTokenType

`func (o *ClientToken) SetTokenType(v string)`

SetTokenType sets TokenType field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


