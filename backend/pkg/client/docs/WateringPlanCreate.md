# WateringPlanCreate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**Date** | **string** |  | 
**Description** | **string** |  | 
**Provider** | Pointer to **string** |  | [optional] 
**TrailerId** | Pointer to **int32** |  | [optional] 
**TransporterId** | **int32** |  | 
**TreeClusterIds** | **[]int32** |  | 
**UserIds** | **[]string** |  | 

## Methods

### NewWateringPlanCreate

`func NewWateringPlanCreate(date string, description string, transporterId int32, treeClusterIds []int32, userIds []string, ) *WateringPlanCreate`

NewWateringPlanCreate instantiates a new WateringPlanCreate object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewWateringPlanCreateWithDefaults

`func NewWateringPlanCreateWithDefaults() *WateringPlanCreate`

NewWateringPlanCreateWithDefaults instantiates a new WateringPlanCreate object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *WateringPlanCreate) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *WateringPlanCreate) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *WateringPlanCreate) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *WateringPlanCreate) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetDate

`func (o *WateringPlanCreate) GetDate() string`

GetDate returns the Date field if non-nil, zero value otherwise.

### GetDateOk

`func (o *WateringPlanCreate) GetDateOk() (*string, bool)`

GetDateOk returns a tuple with the Date field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDate

`func (o *WateringPlanCreate) SetDate(v string)`

SetDate sets Date field to given value.


### GetDescription

`func (o *WateringPlanCreate) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *WateringPlanCreate) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *WateringPlanCreate) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetProvider

`func (o *WateringPlanCreate) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *WateringPlanCreate) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *WateringPlanCreate) SetProvider(v string)`

SetProvider sets Provider field to given value.

### HasProvider

`func (o *WateringPlanCreate) HasProvider() bool`

HasProvider returns a boolean if a field has been set.

### GetTrailerId

`func (o *WateringPlanCreate) GetTrailerId() int32`

GetTrailerId returns the TrailerId field if non-nil, zero value otherwise.

### GetTrailerIdOk

`func (o *WateringPlanCreate) GetTrailerIdOk() (*int32, bool)`

GetTrailerIdOk returns a tuple with the TrailerId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrailerId

`func (o *WateringPlanCreate) SetTrailerId(v int32)`

SetTrailerId sets TrailerId field to given value.

### HasTrailerId

`func (o *WateringPlanCreate) HasTrailerId() bool`

HasTrailerId returns a boolean if a field has been set.

### GetTransporterId

`func (o *WateringPlanCreate) GetTransporterId() int32`

GetTransporterId returns the TransporterId field if non-nil, zero value otherwise.

### GetTransporterIdOk

`func (o *WateringPlanCreate) GetTransporterIdOk() (*int32, bool)`

GetTransporterIdOk returns a tuple with the TransporterId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTransporterId

`func (o *WateringPlanCreate) SetTransporterId(v int32)`

SetTransporterId sets TransporterId field to given value.


### GetTreeClusterIds

`func (o *WateringPlanCreate) GetTreeClusterIds() []int32`

GetTreeClusterIds returns the TreeClusterIds field if non-nil, zero value otherwise.

### GetTreeClusterIdsOk

`func (o *WateringPlanCreate) GetTreeClusterIdsOk() (*[]int32, bool)`

GetTreeClusterIdsOk returns a tuple with the TreeClusterIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeClusterIds

`func (o *WateringPlanCreate) SetTreeClusterIds(v []int32)`

SetTreeClusterIds sets TreeClusterIds field to given value.


### GetUserIds

`func (o *WateringPlanCreate) GetUserIds() []string`

GetUserIds returns the UserIds field if non-nil, zero value otherwise.

### GetUserIdsOk

`func (o *WateringPlanCreate) GetUserIdsOk() (*[]string, bool)`

GetUserIdsOk returns a tuple with the UserIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUserIds

`func (o *WateringPlanCreate) SetUserIds(v []string)`

SetUserIds sets UserIds field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


