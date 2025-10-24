# PluginRegisterRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Auth** | [**PluginAuth**](PluginAuth.md) |  | 
**Description** | **string** |  | 
**Name** | **string** |  | 
**Path** | **string** |  | 
**Slug** | **string** |  | 
**Version** | **string** |  | 

## Methods

### NewPluginRegisterRequest

`func NewPluginRegisterRequest(auth PluginAuth, description string, name string, path string, slug string, version string, ) *PluginRegisterRequest`

NewPluginRegisterRequest instantiates a new PluginRegisterRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPluginRegisterRequestWithDefaults

`func NewPluginRegisterRequestWithDefaults() *PluginRegisterRequest`

NewPluginRegisterRequestWithDefaults instantiates a new PluginRegisterRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAuth

`func (o *PluginRegisterRequest) GetAuth() PluginAuth`

GetAuth returns the Auth field if non-nil, zero value otherwise.

### GetAuthOk

`func (o *PluginRegisterRequest) GetAuthOk() (*PluginAuth, bool)`

GetAuthOk returns a tuple with the Auth field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAuth

`func (o *PluginRegisterRequest) SetAuth(v PluginAuth)`

SetAuth sets Auth field to given value.


### GetDescription

`func (o *PluginRegisterRequest) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *PluginRegisterRequest) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *PluginRegisterRequest) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetName

`func (o *PluginRegisterRequest) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *PluginRegisterRequest) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *PluginRegisterRequest) SetName(v string)`

SetName sets Name field to given value.


### GetPath

`func (o *PluginRegisterRequest) GetPath() string`

GetPath returns the Path field if non-nil, zero value otherwise.

### GetPathOk

`func (o *PluginRegisterRequest) GetPathOk() (*string, bool)`

GetPathOk returns a tuple with the Path field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPath

`func (o *PluginRegisterRequest) SetPath(v string)`

SetPath sets Path field to given value.


### GetSlug

`func (o *PluginRegisterRequest) GetSlug() string`

GetSlug returns the Slug field if non-nil, zero value otherwise.

### GetSlugOk

`func (o *PluginRegisterRequest) GetSlugOk() (*string, bool)`

GetSlugOk returns a tuple with the Slug field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSlug

`func (o *PluginRegisterRequest) SetSlug(v string)`

SetSlug sets Slug field to given value.


### GetVersion

`func (o *PluginRegisterRequest) GetVersion() string`

GetVersion returns the Version field if non-nil, zero value otherwise.

### GetVersionOk

`func (o *PluginRegisterRequest) GetVersionOk() (*string, bool)`

GetVersionOk returns a tuple with the Version field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVersion

`func (o *PluginRegisterRequest) SetVersion(v string)`

SetVersion sets Version field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


