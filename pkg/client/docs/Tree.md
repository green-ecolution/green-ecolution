# Tree

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**CreatedAt** | **string** |  | 
**Description** | **string** |  | 
**Id** | **int32** |  | 
**LastWatered** | Pointer to **string** |  | [optional] 
**Latitude** | **float32** |  | 
**Longitude** | **float32** |  | 
**Number** | **string** |  | 
**PlantingYear** | **int32** |  | 
**Provider** | **string** |  | 
**Sensor** | Pointer to [**Sensor**](Sensor.md) |  | [optional] 
**Species** | **string** |  | 
**TreeClusterId** | Pointer to **int32** |  | [optional] 
**UpdatedAt** | **string** |  | 
**WateringStatus** | [**WateringStatus**](WateringStatus.md) |  | 

## Methods

### NewTree

`func NewTree(createdAt string, description string, id int32, latitude float32, longitude float32, number string, plantingYear int32, provider string, species string, updatedAt string, wateringStatus WateringStatus, ) *Tree`

NewTree instantiates a new Tree object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTreeWithDefaults

`func NewTreeWithDefaults() *Tree`

NewTreeWithDefaults instantiates a new Tree object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *Tree) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *Tree) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *Tree) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *Tree) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetCreatedAt

`func (o *Tree) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *Tree) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *Tree) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetDescription

`func (o *Tree) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *Tree) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *Tree) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetId

`func (o *Tree) GetId() int32`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *Tree) GetIdOk() (*int32, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *Tree) SetId(v int32)`

SetId sets Id field to given value.


### GetLastWatered

`func (o *Tree) GetLastWatered() string`

GetLastWatered returns the LastWatered field if non-nil, zero value otherwise.

### GetLastWateredOk

`func (o *Tree) GetLastWateredOk() (*string, bool)`

GetLastWateredOk returns a tuple with the LastWatered field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLastWatered

`func (o *Tree) SetLastWatered(v string)`

SetLastWatered sets LastWatered field to given value.

### HasLastWatered

`func (o *Tree) HasLastWatered() bool`

HasLastWatered returns a boolean if a field has been set.

### GetLatitude

`func (o *Tree) GetLatitude() float32`

GetLatitude returns the Latitude field if non-nil, zero value otherwise.

### GetLatitudeOk

`func (o *Tree) GetLatitudeOk() (*float32, bool)`

GetLatitudeOk returns a tuple with the Latitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLatitude

`func (o *Tree) SetLatitude(v float32)`

SetLatitude sets Latitude field to given value.


### GetLongitude

`func (o *Tree) GetLongitude() float32`

GetLongitude returns the Longitude field if non-nil, zero value otherwise.

### GetLongitudeOk

`func (o *Tree) GetLongitudeOk() (*float32, bool)`

GetLongitudeOk returns a tuple with the Longitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLongitude

`func (o *Tree) SetLongitude(v float32)`

SetLongitude sets Longitude field to given value.


### GetNumber

`func (o *Tree) GetNumber() string`

GetNumber returns the Number field if non-nil, zero value otherwise.

### GetNumberOk

`func (o *Tree) GetNumberOk() (*string, bool)`

GetNumberOk returns a tuple with the Number field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNumber

`func (o *Tree) SetNumber(v string)`

SetNumber sets Number field to given value.


### GetPlantingYear

`func (o *Tree) GetPlantingYear() int32`

GetPlantingYear returns the PlantingYear field if non-nil, zero value otherwise.

### GetPlantingYearOk

`func (o *Tree) GetPlantingYearOk() (*int32, bool)`

GetPlantingYearOk returns a tuple with the PlantingYear field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPlantingYear

`func (o *Tree) SetPlantingYear(v int32)`

SetPlantingYear sets PlantingYear field to given value.


### GetProvider

`func (o *Tree) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *Tree) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *Tree) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetSensor

`func (o *Tree) GetSensor() Sensor`

GetSensor returns the Sensor field if non-nil, zero value otherwise.

### GetSensorOk

`func (o *Tree) GetSensorOk() (*Sensor, bool)`

GetSensorOk returns a tuple with the Sensor field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSensor

`func (o *Tree) SetSensor(v Sensor)`

SetSensor sets Sensor field to given value.

### HasSensor

`func (o *Tree) HasSensor() bool`

HasSensor returns a boolean if a field has been set.

### GetSpecies

`func (o *Tree) GetSpecies() string`

GetSpecies returns the Species field if non-nil, zero value otherwise.

### GetSpeciesOk

`func (o *Tree) GetSpeciesOk() (*string, bool)`

GetSpeciesOk returns a tuple with the Species field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpecies

`func (o *Tree) SetSpecies(v string)`

SetSpecies sets Species field to given value.


### GetTreeClusterId

`func (o *Tree) GetTreeClusterId() int32`

GetTreeClusterId returns the TreeClusterId field if non-nil, zero value otherwise.

### GetTreeClusterIdOk

`func (o *Tree) GetTreeClusterIdOk() (*int32, bool)`

GetTreeClusterIdOk returns a tuple with the TreeClusterId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeClusterId

`func (o *Tree) SetTreeClusterId(v int32)`

SetTreeClusterId sets TreeClusterId field to given value.

### HasTreeClusterId

`func (o *Tree) HasTreeClusterId() bool`

HasTreeClusterId returns a boolean if a field has been set.

### GetUpdatedAt

`func (o *Tree) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *Tree) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *Tree) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.


### GetWateringStatus

`func (o *Tree) GetWateringStatus() WateringStatus`

GetWateringStatus returns the WateringStatus field if non-nil, zero value otherwise.

### GetWateringStatusOk

`func (o *Tree) GetWateringStatusOk() (*WateringStatus, bool)`

GetWateringStatusOk returns a tuple with the WateringStatus field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWateringStatus

`func (o *Tree) SetWateringStatus(v WateringStatus)`

SetWateringStatus sets WateringStatus field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


