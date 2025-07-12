# SensorData

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Battery** | **float32** |  | 
**CreatedAt** | **string** |  | 
**Humidity** | **float32** |  | 
**Temperature** | **float32** |  | 
**UpdatedAt** | **string** |  | 
**Watermarks** | [**[]WatermarkResponse**](WatermarkResponse.md) |  | 

## Methods

### NewSensorData

`func NewSensorData(battery float32, createdAt string, humidity float32, temperature float32, updatedAt string, watermarks []WatermarkResponse, ) *SensorData`

NewSensorData instantiates a new SensorData object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewSensorDataWithDefaults

`func NewSensorDataWithDefaults() *SensorData`

NewSensorDataWithDefaults instantiates a new SensorData object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBattery

`func (o *SensorData) GetBattery() float32`

GetBattery returns the Battery field if non-nil, zero value otherwise.

### GetBatteryOk

`func (o *SensorData) GetBatteryOk() (*float32, bool)`

GetBatteryOk returns a tuple with the Battery field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBattery

`func (o *SensorData) SetBattery(v float32)`

SetBattery sets Battery field to given value.


### GetCreatedAt

`func (o *SensorData) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *SensorData) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *SensorData) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetHumidity

`func (o *SensorData) GetHumidity() float32`

GetHumidity returns the Humidity field if non-nil, zero value otherwise.

### GetHumidityOk

`func (o *SensorData) GetHumidityOk() (*float32, bool)`

GetHumidityOk returns a tuple with the Humidity field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHumidity

`func (o *SensorData) SetHumidity(v float32)`

SetHumidity sets Humidity field to given value.


### GetTemperature

`func (o *SensorData) GetTemperature() float32`

GetTemperature returns the Temperature field if non-nil, zero value otherwise.

### GetTemperatureOk

`func (o *SensorData) GetTemperatureOk() (*float32, bool)`

GetTemperatureOk returns a tuple with the Temperature field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTemperature

`func (o *SensorData) SetTemperature(v float32)`

SetTemperature sets Temperature field to given value.


### GetUpdatedAt

`func (o *SensorData) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *SensorData) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *SensorData) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.


### GetWatermarks

`func (o *SensorData) GetWatermarks() []WatermarkResponse`

GetWatermarks returns the Watermarks field if non-nil, zero value otherwise.

### GetWatermarksOk

`func (o *SensorData) GetWatermarksOk() (*[]WatermarkResponse, bool)`

GetWatermarksOk returns a tuple with the Watermarks field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWatermarks

`func (o *SensorData) SetWatermarks(v []WatermarkResponse)`

SetWatermarks sets Watermarks field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


