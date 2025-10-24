# Sensor

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**CreatedAt** | **string** |  | 
**Id** | **string** |  | 
**LatestData** | [**SensorData**](SensorData.md) |  | 
**Latitude** | **float32** |  | 
**Longitude** | **float32** |  | 
**Provider** | Pointer to **string** |  | [optional] 
**Status** | [**SensorStatus**](SensorStatus.md) |  | 
**UpdatedAt** | **string** |  | 

## Methods

### NewSensor

`func NewSensor(createdAt string, id string, latestData SensorData, latitude float32, longitude float32, status SensorStatus, updatedAt string, ) *Sensor`

NewSensor instantiates a new Sensor object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewSensorWithDefaults

`func NewSensorWithDefaults() *Sensor`

NewSensorWithDefaults instantiates a new Sensor object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *Sensor) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *Sensor) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *Sensor) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *Sensor) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetCreatedAt

`func (o *Sensor) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *Sensor) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *Sensor) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetId

`func (o *Sensor) GetId() string`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *Sensor) GetIdOk() (*string, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *Sensor) SetId(v string)`

SetId sets Id field to given value.


### GetLatestData

`func (o *Sensor) GetLatestData() SensorData`

GetLatestData returns the LatestData field if non-nil, zero value otherwise.

### GetLatestDataOk

`func (o *Sensor) GetLatestDataOk() (*SensorData, bool)`

GetLatestDataOk returns a tuple with the LatestData field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLatestData

`func (o *Sensor) SetLatestData(v SensorData)`

SetLatestData sets LatestData field to given value.


### GetLatitude

`func (o *Sensor) GetLatitude() float32`

GetLatitude returns the Latitude field if non-nil, zero value otherwise.

### GetLatitudeOk

`func (o *Sensor) GetLatitudeOk() (*float32, bool)`

GetLatitudeOk returns a tuple with the Latitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLatitude

`func (o *Sensor) SetLatitude(v float32)`

SetLatitude sets Latitude field to given value.


### GetLongitude

`func (o *Sensor) GetLongitude() float32`

GetLongitude returns the Longitude field if non-nil, zero value otherwise.

### GetLongitudeOk

`func (o *Sensor) GetLongitudeOk() (*float32, bool)`

GetLongitudeOk returns a tuple with the Longitude field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLongitude

`func (o *Sensor) SetLongitude(v float32)`

SetLongitude sets Longitude field to given value.


### GetProvider

`func (o *Sensor) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *Sensor) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *Sensor) SetProvider(v string)`

SetProvider sets Provider field to given value.

### HasProvider

`func (o *Sensor) HasProvider() bool`

HasProvider returns a boolean if a field has been set.

### GetStatus

`func (o *Sensor) GetStatus() SensorStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *Sensor) GetStatusOk() (*SensorStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *Sensor) SetStatus(v SensorStatus)`

SetStatus sets Status field to given value.


### GetUpdatedAt

`func (o *Sensor) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *Sensor) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *Sensor) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


