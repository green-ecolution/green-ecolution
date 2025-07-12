# TreeClusterInList

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**Address** | **string** |  | 
**Archived** | **bool** |  | 
**CreatedAt** | **string** |  | 
**Description** | **string** |  | 
**Id** | **int32** |  | 
**LastWatered** | Pointer to **string** |  | [optional] 
**Latitude** | **float32** |  | 
**Longitude** | **float32** |  | 
**MoistureLevel** | **float32** |  | 
**Name** | **string** |  | 
**Provider** | **string** |  | 
**Region** | Pointer to [**Region**](Region.md) |  | [optional] 
**SoilCondition** | [**SoilCondition**](SoilCondition.md) |  | 
**TreeIds** | Pointer to **[]int32** |  | [optional] 
**UpdatedAt** | **string** |  | 
**WateringStatus** | [**WateringStatus**](WateringStatus.md) |  | 

## Methods

### NewTreeClusterInList

`func NewTreeClusterInList(address string, archived bool, createdAt string, description string, id int32, latitude float32, longitude float32, moistureLevel float32, name string, provider string, soilCondition SoilCondition, updatedAt string, wateringStatus WateringStatus, ) *TreeClusterInList`

NewTreeClusterInList instantiates a new TreeClusterInList object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTreeClusterInListWithDefaults

`func NewTreeClusterInListWithDefaults() *TreeClusterInList`

NewTreeClusterInListWithDefaults instantiates a new TreeClusterInList object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *TreeClusterInList) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *TreeClusterInList) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *TreeClusterInList) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *TreeClusterInList) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetAddress

`func (o *TreeClusterInList) GetAddress() string`

GetAddress returns the Address field if non-nil, zero value otherwise.

### GetAddressOk

`func (o *TreeClusterInList) GetAddressOk() (*string, bool)`

GetAddressOk returns a tuple with the Address field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAddress

`func (o *TreeClusterInList) SetAddress(v string)`

SetAddress sets Address field to given value.


### GetArchived

`func (o *TreeClusterInList) GetArchived() bool`

GetArchived returns the Archived field if non-nil, zero value otherwise.

### GetArchivedOk

`func (o *TreeClusterInList) GetArchivedOk() (*bool, bool)`

GetArchivedOk returns a tuple with the Archived field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetArchived

`func (o *TreeClusterInList) SetArchived(v bool)`

SetArchived sets Archived field to given value.


### GetCreatedAt

`func (o *TreeClusterInList) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *TreeClusterInList) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *TreeClusterInList) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetDescription

`func (o *TreeClusterInList) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *TreeClusterInList) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *TreeClusterInList) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetId

`func (o *TreeClusterInList) GetId() int32`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *TreeClusterInList) GetIdOk() (*int32, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *TreeClusterInList) SetId(v int32)`

SetId sets Id field to given value.


### GetLastWatered

`func (o *TreeClusterInList) GetLastWatered() string`

GetLastWatered returns the LastWatered field if non-nil, zero value otherwise.

### GetLastWateredOk

`func (o *TreeClusterInList) GetLastWateredOk() (*string, bool)`

GetLastWateredOk returns a tuple with the LastWatered field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLastWatered

`func (o *TreeClusterInList) SetLastWatered(v string)`

SetLastWatered sets LastWatered field to given value.

### HasLastWatered

`func (o *TreeClusterInList) HasLastWatered() bool`

HasLastWatered returns a boolean if a field has been set.

### GetLatitude

`func (o *TreeClusterInList) GetLatitude() float32`

GetLatitude returns the Latitude field if non-nil, zero value otherwise.

### GetLatitudeOk

`func (o *TreeClusterInList) GetLatitudeOk() (*float32, bool)`

GetLatitudeOk returns a tuple with the Latitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLatitude

`func (o *TreeClusterInList) SetLatitude(v float32)`

SetLatitude sets Latitude field to given value.


### GetLongitude

`func (o *TreeClusterInList) GetLongitude() float32`

GetLongitude returns the Longitude field if non-nil, zero value otherwise.

### GetLongitudeOk

`func (o *TreeClusterInList) GetLongitudeOk() (*float32, bool)`

GetLongitudeOk returns a tuple with the Longitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLongitude

`func (o *TreeClusterInList) SetLongitude(v float32)`

SetLongitude sets Longitude field to given value.


### GetMoistureLevel

`func (o *TreeClusterInList) GetMoistureLevel() float32`

GetMoistureLevel returns the MoistureLevel field if non-nil, zero value otherwise.

### GetMoistureLevelOk

`func (o *TreeClusterInList) GetMoistureLevelOk() (*float32, bool)`

GetMoistureLevelOk returns a tuple with the MoistureLevel field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMoistureLevel

`func (o *TreeClusterInList) SetMoistureLevel(v float32)`

SetMoistureLevel sets MoistureLevel field to given value.


### GetName

`func (o *TreeClusterInList) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *TreeClusterInList) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *TreeClusterInList) SetName(v string)`

SetName sets Name field to given value.


### GetProvider

`func (o *TreeClusterInList) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *TreeClusterInList) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *TreeClusterInList) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetRegion

`func (o *TreeClusterInList) GetRegion() Region`

GetRegion returns the Region field if non-nil, zero value otherwise.

### GetRegionOk

`func (o *TreeClusterInList) GetRegionOk() (*Region, bool)`

GetRegionOk returns a tuple with the Region field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRegion

`func (o *TreeClusterInList) SetRegion(v Region)`

SetRegion sets Region field to given value.

### HasRegion

`func (o *TreeClusterInList) HasRegion() bool`

HasRegion returns a boolean if a field has been set.

### GetSoilCondition

`func (o *TreeClusterInList) GetSoilCondition() SoilCondition`

GetSoilCondition returns the SoilCondition field if non-nil, zero value otherwise.

### GetSoilConditionOk

`func (o *TreeClusterInList) GetSoilConditionOk() (*SoilCondition, bool)`

GetSoilConditionOk returns a tuple with the SoilCondition field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSoilCondition

`func (o *TreeClusterInList) SetSoilCondition(v SoilCondition)`

SetSoilCondition sets SoilCondition field to given value.


### GetTreeIds

`func (o *TreeClusterInList) GetTreeIds() []int32`

GetTreeIds returns the TreeIds field if non-nil, zero value otherwise.

### GetTreeIdsOk

`func (o *TreeClusterInList) GetTreeIdsOk() (*[]int32, bool)`

GetTreeIdsOk returns a tuple with the TreeIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeIds

`func (o *TreeClusterInList) SetTreeIds(v []int32)`

SetTreeIds sets TreeIds field to given value.

### HasTreeIds

`func (o *TreeClusterInList) HasTreeIds() bool`

HasTreeIds returns a boolean if a field has been set.

### GetUpdatedAt

`func (o *TreeClusterInList) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *TreeClusterInList) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *TreeClusterInList) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.


### GetWateringStatus

`func (o *TreeClusterInList) GetWateringStatus() WateringStatus`

GetWateringStatus returns the WateringStatus field if non-nil, zero value otherwise.

### GetWateringStatusOk

`func (o *TreeClusterInList) GetWateringStatusOk() (*WateringStatus, bool)`

GetWateringStatusOk returns a tuple with the WateringStatus field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWateringStatus

`func (o *TreeClusterInList) SetWateringStatus(v WateringStatus)`

SetWateringStatus sets WateringStatus field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


