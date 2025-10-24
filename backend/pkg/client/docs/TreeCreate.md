# TreeCreate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**Description** | **string** |  | 
**Latitude** | **float32** |  | 
**Longitude** | **float32** |  | 
**Number** | **string** |  | 
**PlantingYear** | **int32** |  | 
**Provider** | Pointer to **string** |  | [optional] 
**SensorId** | Pointer to **string** |  | [optional] 
**Species** | **string** |  | 
**TreeClusterId** | Pointer to **int32** |  | [optional] 

## Methods

### NewTreeCreate

`func NewTreeCreate(description string, latitude float32, longitude float32, number string, plantingYear int32, species string, ) *TreeCreate`

NewTreeCreate instantiates a new TreeCreate object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTreeCreateWithDefaults

`func NewTreeCreateWithDefaults() *TreeCreate`

NewTreeCreateWithDefaults instantiates a new TreeCreate object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *TreeCreate) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *TreeCreate) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *TreeCreate) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *TreeCreate) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetDescription

`func (o *TreeCreate) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *TreeCreate) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *TreeCreate) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetLatitude

`func (o *TreeCreate) GetLatitude() float32`

GetLatitude returns the Latitude field if non-nil, zero value otherwise.

### GetLatitudeOk

`func (o *TreeCreate) GetLatitudeOk() (*float32, bool)`

GetLatitudeOk returns a tuple with the Latitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLatitude

`func (o *TreeCreate) SetLatitude(v float32)`

SetLatitude sets Latitude field to given value.


### GetLongitude

`func (o *TreeCreate) GetLongitude() float32`

GetLongitude returns the Longitude field if non-nil, zero value otherwise.

### GetLongitudeOk

`func (o *TreeCreate) GetLongitudeOk() (*float32, bool)`

GetLongitudeOk returns a tuple with the Longitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLongitude

`func (o *TreeCreate) SetLongitude(v float32)`

SetLongitude sets Longitude field to given value.


### GetNumber

`func (o *TreeCreate) GetNumber() string`

GetNumber returns the Number field if non-nil, zero value otherwise.

### GetNumberOk

`func (o *TreeCreate) GetNumberOk() (*string, bool)`

GetNumberOk returns a tuple with the Number field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNumber

`func (o *TreeCreate) SetNumber(v string)`

SetNumber sets Number field to given value.


### GetPlantingYear

`func (o *TreeCreate) GetPlantingYear() int32`

GetPlantingYear returns the PlantingYear field if non-nil, zero value otherwise.

### GetPlantingYearOk

`func (o *TreeCreate) GetPlantingYearOk() (*int32, bool)`

GetPlantingYearOk returns a tuple with the PlantingYear field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPlantingYear

`func (o *TreeCreate) SetPlantingYear(v int32)`

SetPlantingYear sets PlantingYear field to given value.


### GetProvider

`func (o *TreeCreate) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *TreeCreate) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *TreeCreate) SetProvider(v string)`

SetProvider sets Provider field to given value.

### HasProvider

`func (o *TreeCreate) HasProvider() bool`

HasProvider returns a boolean if a field has been set.

### GetSensorId

`func (o *TreeCreate) GetSensorId() string`

GetSensorId returns the SensorId field if non-nil, zero value otherwise.

### GetSensorIdOk

`func (o *TreeCreate) GetSensorIdOk() (*string, bool)`

GetSensorIdOk returns a tuple with the SensorId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSensorId

`func (o *TreeCreate) SetSensorId(v string)`

SetSensorId sets SensorId field to given value.

### HasSensorId

`func (o *TreeCreate) HasSensorId() bool`

HasSensorId returns a boolean if a field has been set.

### GetSpecies

`func (o *TreeCreate) GetSpecies() string`

GetSpecies returns the Species field if non-nil, zero value otherwise.

### GetSpeciesOk

`func (o *TreeCreate) GetSpeciesOk() (*string, bool)`

GetSpeciesOk returns a tuple with the Species field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpecies

`func (o *TreeCreate) SetSpecies(v string)`

SetSpecies sets Species field to given value.


### GetTreeClusterId

`func (o *TreeCreate) GetTreeClusterId() int32`

GetTreeClusterId returns the TreeClusterId field if non-nil, zero value otherwise.

### GetTreeClusterIdOk

`func (o *TreeCreate) GetTreeClusterIdOk() (*int32, bool)`

GetTreeClusterIdOk returns a tuple with the TreeClusterId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeClusterId

`func (o *TreeCreate) SetTreeClusterId(v int32)`

SetTreeClusterId sets TreeClusterId field to given value.

### HasTreeClusterId

`func (o *TreeCreate) HasTreeClusterId() bool`

HasTreeClusterId returns a boolean if a field has been set.


[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


