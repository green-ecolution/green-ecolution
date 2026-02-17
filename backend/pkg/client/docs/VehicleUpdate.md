# VehicleUpdate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | **map[string]interface{}** |  | 
**Description** | **string** |  | 
**DrivingLicense** | [**DrivingLicense**](DrivingLicense.md) |  | 
**Height** | **float32** |  | 
**Length** | **float32** |  | 
**Model** | **string** |  | 
**NumberPlate** | **string** |  | 
**Provider** | **string** |  | 
**Status** | [**VehicleStatus**](VehicleStatus.md) |  | 
**Type** | [**VehicleType**](VehicleType.md) |  | 
**WaterCapacity** | **float32** |  | 
**Weight** | **float32** |  | 
**Width** | **float32** |  | 

## Methods

### NewVehicleUpdate

`func NewVehicleUpdate(additionalInformation map[string]interface{}, description string, drivingLicense DrivingLicense, height float32, length float32, model string, numberPlate string, provider string, status VehicleStatus, type_ VehicleType, waterCapacity float32, weight float32, width float32, ) *VehicleUpdate`

NewVehicleUpdate instantiates a new VehicleUpdate object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewVehicleUpdateWithDefaults

`func NewVehicleUpdateWithDefaults() *VehicleUpdate`

NewVehicleUpdateWithDefaults instantiates a new VehicleUpdate object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *VehicleUpdate) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *VehicleUpdate) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *VehicleUpdate) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.


### GetDescription

`func (o *VehicleUpdate) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *VehicleUpdate) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *VehicleUpdate) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetDrivingLicense

`func (o *VehicleUpdate) GetDrivingLicense() DrivingLicense`

GetDrivingLicense returns the DrivingLicense field if non-nil, zero value otherwise.

### GetDrivingLicenseOk

`func (o *VehicleUpdate) GetDrivingLicenseOk() (*DrivingLicense, bool)`

GetDrivingLicenseOk returns a tuple with the DrivingLicense field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDrivingLicense

`func (o *VehicleUpdate) SetDrivingLicense(v DrivingLicense)`

SetDrivingLicense sets DrivingLicense field to given value.


### GetHeight

`func (o *VehicleUpdate) GetHeight() float32`

GetHeight returns the Height field if non-nil, zero value otherwise.

### GetHeightOk

`func (o *VehicleUpdate) GetHeightOk() (*float32, bool)`

GetHeightOk returns a tuple with the Height field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHeight

`func (o *VehicleUpdate) SetHeight(v float32)`

SetHeight sets Height field to given value.


### GetLength

`func (o *VehicleUpdate) GetLength() float32`

GetLength returns the Length field if non-nil, zero value otherwise.

### GetLengthOk

`func (o *VehicleUpdate) GetLengthOk() (*float32, bool)`

GetLengthOk returns a tuple with the Length field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLength

`func (o *VehicleUpdate) SetLength(v float32)`

SetLength sets Length field to given value.


### GetModel

`func (o *VehicleUpdate) GetModel() string`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *VehicleUpdate) GetModelOk() (*string, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *VehicleUpdate) SetModel(v string)`

SetModel sets Model field to given value.


### GetNumberPlate

`func (o *VehicleUpdate) GetNumberPlate() string`

GetNumberPlate returns the NumberPlate field if non-nil, zero value otherwise.

### GetNumberPlateOk

`func (o *VehicleUpdate) GetNumberPlateOk() (*string, bool)`

GetNumberPlateOk returns a tuple with the NumberPlate field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNumberPlate

`func (o *VehicleUpdate) SetNumberPlate(v string)`

SetNumberPlate sets NumberPlate field to given value.


### GetProvider

`func (o *VehicleUpdate) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *VehicleUpdate) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *VehicleUpdate) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetStatus

`func (o *VehicleUpdate) GetStatus() VehicleStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *VehicleUpdate) GetStatusOk() (*VehicleStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *VehicleUpdate) SetStatus(v VehicleStatus)`

SetStatus sets Status field to given value.


### GetType

`func (o *VehicleUpdate) GetType() VehicleType`

GetType returns the Type field if non-nil, zero value otherwise.

### GetTypeOk

`func (o *VehicleUpdate) GetTypeOk() (*VehicleType, bool)`

GetTypeOk returns a tuple with the Type field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetType

`func (o *VehicleUpdate) SetType(v VehicleType)`

SetType sets Type field to given value.


### GetWaterCapacity

`func (o *VehicleUpdate) GetWaterCapacity() float32`

GetWaterCapacity returns the WaterCapacity field if non-nil, zero value otherwise.

### GetWaterCapacityOk

`func (o *VehicleUpdate) GetWaterCapacityOk() (*float32, bool)`

GetWaterCapacityOk returns a tuple with the WaterCapacity field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWaterCapacity

`func (o *VehicleUpdate) SetWaterCapacity(v float32)`

SetWaterCapacity sets WaterCapacity field to given value.


### GetWeight

`func (o *VehicleUpdate) GetWeight() float32`

GetWeight returns the Weight field if non-nil, zero value otherwise.

### GetWeightOk

`func (o *VehicleUpdate) GetWeightOk() (*float32, bool)`

GetWeightOk returns a tuple with the Weight field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWeight

`func (o *VehicleUpdate) SetWeight(v float32)`

SetWeight sets Weight field to given value.


### GetWidth

`func (o *VehicleUpdate) GetWidth() float32`

GetWidth returns the Width field if non-nil, zero value otherwise.

### GetWidthOk

`func (o *VehicleUpdate) GetWidthOk() (*float32, bool)`

GetWidthOk returns a tuple with the Width field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWidth

`func (o *VehicleUpdate) SetWidth(v float32)`

SetWidth sets Width field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


