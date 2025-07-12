# TreeCluster

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
**Trees** | Pointer to [**[]Tree**](Tree.md) |  | [optional] 
**UpdatedAt** | **string** |  | 
**WateringStatus** | [**WateringStatus**](WateringStatus.md) |  | 

## Methods

### NewTreeCluster

`func NewTreeCluster(address string, archived bool, createdAt string, description string, id int32, latitude float32, longitude float32, moistureLevel float32, name string, provider string, soilCondition SoilCondition, updatedAt string, wateringStatus WateringStatus, ) *TreeCluster`

NewTreeCluster instantiates a new TreeCluster object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTreeClusterWithDefaults

`func NewTreeClusterWithDefaults() *TreeCluster`

NewTreeClusterWithDefaults instantiates a new TreeCluster object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *TreeCluster) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *TreeCluster) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *TreeCluster) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *TreeCluster) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetAddress

`func (o *TreeCluster) GetAddress() string`

GetAddress returns the Address field if non-nil, zero value otherwise.

### GetAddressOk

`func (o *TreeCluster) GetAddressOk() (*string, bool)`

GetAddressOk returns a tuple with the Address field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAddress

`func (o *TreeCluster) SetAddress(v string)`

SetAddress sets Address field to given value.


### GetArchived

`func (o *TreeCluster) GetArchived() bool`

GetArchived returns the Archived field if non-nil, zero value otherwise.

### GetArchivedOk

`func (o *TreeCluster) GetArchivedOk() (*bool, bool)`

GetArchivedOk returns a tuple with the Archived field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetArchived

`func (o *TreeCluster) SetArchived(v bool)`

SetArchived sets Archived field to given value.


### GetCreatedAt

`func (o *TreeCluster) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *TreeCluster) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *TreeCluster) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetDescription

`func (o *TreeCluster) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *TreeCluster) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *TreeCluster) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetId

`func (o *TreeCluster) GetId() int32`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *TreeCluster) GetIdOk() (*int32, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *TreeCluster) SetId(v int32)`

SetId sets Id field to given value.


### GetLastWatered

`func (o *TreeCluster) GetLastWatered() string`

GetLastWatered returns the LastWatered field if non-nil, zero value otherwise.

### GetLastWateredOk

`func (o *TreeCluster) GetLastWateredOk() (*string, bool)`

GetLastWateredOk returns a tuple with the LastWatered field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLastWatered

`func (o *TreeCluster) SetLastWatered(v string)`

SetLastWatered sets LastWatered field to given value.

### HasLastWatered

`func (o *TreeCluster) HasLastWatered() bool`

HasLastWatered returns a boolean if a field has been set.

### GetLatitude

`func (o *TreeCluster) GetLatitude() float32`

GetLatitude returns the Latitude field if non-nil, zero value otherwise.

### GetLatitudeOk

`func (o *TreeCluster) GetLatitudeOk() (*float32, bool)`

GetLatitudeOk returns a tuple with the Latitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLatitude

`func (o *TreeCluster) SetLatitude(v float32)`

SetLatitude sets Latitude field to given value.


### GetLongitude

`func (o *TreeCluster) GetLongitude() float32`

GetLongitude returns the Longitude field if non-nil, zero value otherwise.

### GetLongitudeOk

`func (o *TreeCluster) GetLongitudeOk() (*float32, bool)`

GetLongitudeOk returns a tuple with the Longitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLongitude

`func (o *TreeCluster) SetLongitude(v float32)`

SetLongitude sets Longitude field to given value.


### GetMoistureLevel

`func (o *TreeCluster) GetMoistureLevel() float32`

GetMoistureLevel returns the MoistureLevel field if non-nil, zero value otherwise.

### GetMoistureLevelOk

`func (o *TreeCluster) GetMoistureLevelOk() (*float32, bool)`

GetMoistureLevelOk returns a tuple with the MoistureLevel field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMoistureLevel

`func (o *TreeCluster) SetMoistureLevel(v float32)`

SetMoistureLevel sets MoistureLevel field to given value.


### GetName

`func (o *TreeCluster) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *TreeCluster) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *TreeCluster) SetName(v string)`

SetName sets Name field to given value.


### GetProvider

`func (o *TreeCluster) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *TreeCluster) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *TreeCluster) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetRegion

`func (o *TreeCluster) GetRegion() Region`

GetRegion returns the Region field if non-nil, zero value otherwise.

### GetRegionOk

`func (o *TreeCluster) GetRegionOk() (*Region, bool)`

GetRegionOk returns a tuple with the Region field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRegion

`func (o *TreeCluster) SetRegion(v Region)`

SetRegion sets Region field to given value.

### HasRegion

`func (o *TreeCluster) HasRegion() bool`

HasRegion returns a boolean if a field has been set.

### GetSoilCondition

`func (o *TreeCluster) GetSoilCondition() SoilCondition`

GetSoilCondition returns the SoilCondition field if non-nil, zero value otherwise.

### GetSoilConditionOk

`func (o *TreeCluster) GetSoilConditionOk() (*SoilCondition, bool)`

GetSoilConditionOk returns a tuple with the SoilCondition field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSoilCondition

`func (o *TreeCluster) SetSoilCondition(v SoilCondition)`

SetSoilCondition sets SoilCondition field to given value.


### GetTrees

`func (o *TreeCluster) GetTrees() []Tree`

GetTrees returns the Trees field if non-nil, zero value otherwise.

### GetTreesOk

`func (o *TreeCluster) GetTreesOk() (*[]Tree, bool)`

GetTreesOk returns a tuple with the Trees field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrees

`func (o *TreeCluster) SetTrees(v []Tree)`

SetTrees sets Trees field to given value.

### HasTrees

`func (o *TreeCluster) HasTrees() bool`

HasTrees returns a boolean if a field has been set.

### GetUpdatedAt

`func (o *TreeCluster) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *TreeCluster) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *TreeCluster) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.


### GetWateringStatus

`func (o *TreeCluster) GetWateringStatus() WateringStatus`

GetWateringStatus returns the WateringStatus field if non-nil, zero value otherwise.

### GetWateringStatusOk

`func (o *TreeCluster) GetWateringStatusOk() (*WateringStatus, bool)`

GetWateringStatusOk returns a tuple with the WateringStatus field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWateringStatus

`func (o *TreeCluster) SetWateringStatus(v WateringStatus)`

SetWateringStatus sets WateringStatus field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


