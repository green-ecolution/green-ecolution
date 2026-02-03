# VersionInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Current** | **string** |  | 
**IsDevelopment** | **bool** |  | 
**IsStage** | **bool** |  | 
**Latest** | Pointer to **string** |  | [optional] 
**ReleaseUrl** | Pointer to **string** |  | [optional] 
**UpdateAvailable** | **bool** |  | 

## Methods

### NewVersionInfo

`func NewVersionInfo(current string, isDevelopment bool, isStage bool, updateAvailable bool, ) *VersionInfo`

NewVersionInfo instantiates a new VersionInfo object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewVersionInfoWithDefaults

`func NewVersionInfoWithDefaults() *VersionInfo`

NewVersionInfoWithDefaults instantiates a new VersionInfo object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCurrent

`func (o *VersionInfo) GetCurrent() string`

GetCurrent returns the Current field if non-nil, zero value otherwise.

### GetCurrentOk

`func (o *VersionInfo) GetCurrentOk() (*string, bool)`

GetCurrentOk returns a tuple with the Current field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCurrent

`func (o *VersionInfo) SetCurrent(v string)`

SetCurrent sets Current field to given value.


### GetIsDevelopment

`func (o *VersionInfo) GetIsDevelopment() bool`

GetIsDevelopment returns the IsDevelopment field if non-nil, zero value otherwise.

### GetIsDevelopmentOk

`func (o *VersionInfo) GetIsDevelopmentOk() (*bool, bool)`

GetIsDevelopmentOk returns a tuple with the IsDevelopment field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetIsDevelopment

`func (o *VersionInfo) SetIsDevelopment(v bool)`

SetIsDevelopment sets IsDevelopment field to given value.


### GetIsStage

`func (o *VersionInfo) GetIsStage() bool`

GetIsStage returns the IsStage field if non-nil, zero value otherwise.

### GetIsStageOk

`func (o *VersionInfo) GetIsStageOk() (*bool, bool)`

GetIsStageOk returns a tuple with the IsStage field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetIsStage

`func (o *VersionInfo) SetIsStage(v bool)`

SetIsStage sets IsStage field to given value.


### GetLatest

`func (o *VersionInfo) GetLatest() string`

GetLatest returns the Latest field if non-nil, zero value otherwise.

### GetLatestOk

`func (o *VersionInfo) GetLatestOk() (*string, bool)`

GetLatestOk returns a tuple with the Latest field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLatest

`func (o *VersionInfo) SetLatest(v string)`

SetLatest sets Latest field to given value.

### HasLatest

`func (o *VersionInfo) HasLatest() bool`

HasLatest returns a boolean if a field has been set.

### GetReleaseUrl

`func (o *VersionInfo) GetReleaseUrl() string`

GetReleaseUrl returns the ReleaseUrl field if non-nil, zero value otherwise.

### GetReleaseUrlOk

`func (o *VersionInfo) GetReleaseUrlOk() (*string, bool)`

GetReleaseUrlOk returns a tuple with the ReleaseUrl field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReleaseUrl

`func (o *VersionInfo) SetReleaseUrl(v string)`

SetReleaseUrl sets ReleaseUrl field to given value.

### HasReleaseUrl

`func (o *VersionInfo) HasReleaseUrl() bool`

HasReleaseUrl returns a boolean if a field has been set.

### GetUpdateAvailable

`func (o *VersionInfo) GetUpdateAvailable() bool`

GetUpdateAvailable returns the UpdateAvailable field if non-nil, zero value otherwise.

### GetUpdateAvailableOk

`func (o *VersionInfo) GetUpdateAvailableOk() (*bool, bool)`

GetUpdateAvailableOk returns a tuple with the UpdateAvailable field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdateAvailable

`func (o *VersionInfo) SetUpdateAvailable(v bool)`

SetUpdateAvailable sets UpdateAvailable field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


