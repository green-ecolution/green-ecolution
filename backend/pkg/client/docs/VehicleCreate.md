# VehicleCreate

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

### NewVehicleCreate

`func NewVehicleCreate(additionalInformation map[string]interface{}, description string, drivingLicense DrivingLicense, height float32, length float32, model string, numberPlate string, provider string, status VehicleStatus, type_ VehicleType, waterCapacity float32, weight float32, width float32, ) *VehicleCreate`

NewVehicleCreate instantiates a new VehicleCreate object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewVehicleCreateWithDefaults

`func NewVehicleCreateWithDefaults() *VehicleCreate`

NewVehicleCreateWithDefaults instantiates a new VehicleCreate object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *VehicleCreate) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *VehicleCreate) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *VehicleCreate) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.


### GetDescription

`func (o *VehicleCreate) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *VehicleCreate) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *VehicleCreate) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetDrivingLicense

`func (o *VehicleCreate) GetDrivingLicense() DrivingLicense`

GetDrivingLicense returns the DrivingLicense field if non-nil, zero value otherwise.

### GetDrivingLicenseOk

`func (o *VehicleCreate) GetDrivingLicenseOk() (*DrivingLicense, bool)`

GetDrivingLicenseOk returns a tuple with the DrivingLicense field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDrivingLicense

`func (o *VehicleCreate) SetDrivingLicense(v DrivingLicense)`

SetDrivingLicense sets DrivingLicense field to given value.


### GetHeight

`func (o *VehicleCreate) GetHeight() float32`

GetHeight returns the Height field if non-nil, zero value otherwise.

### GetHeightOk

`func (o *VehicleCreate) GetHeightOk() (*float32, bool)`

GetHeightOk returns a tuple with the Height field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHeight

`func (o *VehicleCreate) SetHeight(v float32)`

SetHeight sets Height field to given value.


### GetLength

`func (o *VehicleCreate) GetLength() float32`

GetLength returns the Length field if non-nil, zero value otherwise.

### GetLengthOk

`func (o *VehicleCreate) GetLengthOk() (*float32, bool)`

GetLengthOk returns a tuple with the Length field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLength

`func (o *VehicleCreate) SetLength(v float32)`

SetLength sets Length field to given value.


### GetModel

`func (o *VehicleCreate) GetModel() string`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *VehicleCreate) GetModelOk() (*string, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *VehicleCreate) SetModel(v string)`

SetModel sets Model field to given value.


### GetNumberPlate

`func (o *VehicleCreate) GetNumberPlate() string`

GetNumberPlate returns the NumberPlate field if non-nil, zero value otherwise.

### GetNumberPlateOk

`func (o *VehicleCreate) GetNumberPlateOk() (*string, bool)`

GetNumberPlateOk returns a tuple with the NumberPlate field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNumberPlate

`func (o *VehicleCreate) SetNumberPlate(v string)`

SetNumberPlate sets NumberPlate field to given value.


### GetProvider

`func (o *VehicleCreate) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *VehicleCreate) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *VehicleCreate) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetStatus

`func (o *VehicleCreate) GetStatus() VehicleStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *VehicleCreate) GetStatusOk() (*VehicleStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *VehicleCreate) SetStatus(v VehicleStatus)`

SetStatus sets Status field to given value.


### GetType

`func (o *VehicleCreate) GetType() VehicleType`

GetType returns the Type field if non-nil, zero value otherwise.

### GetTypeOk

`func (o *VehicleCreate) GetTypeOk() (*VehicleType, bool)`

GetTypeOk returns a tuple with the Type field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetType

`func (o *VehicleCreate) SetType(v VehicleType)`

SetType sets Type field to given value.


### GetWaterCapacity

`func (o *VehicleCreate) GetWaterCapacity() float32`

GetWaterCapacity returns the WaterCapacity field if non-nil, zero value otherwise.

### GetWaterCapacityOk

`func (o *VehicleCreate) GetWaterCapacityOk() (*float32, bool)`

GetWaterCapacityOk returns a tuple with the WaterCapacity field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWaterCapacity

`func (o *VehicleCreate) SetWaterCapacity(v float32)`

SetWaterCapacity sets WaterCapacity field to given value.


### GetWeight

`func (o *VehicleCreate) GetWeight() float32`

GetWeight returns the Weight field if non-nil, zero value otherwise.

### GetWeightOk

`func (o *VehicleCreate) GetWeightOk() (*float32, bool)`

GetWeightOk returns a tuple with the Weight field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWeight

`func (o *VehicleCreate) SetWeight(v float32)`

SetWeight sets Weight field to given value.


### GetWidth

`func (o *VehicleCreate) GetWidth() float32`

GetWidth returns the Width field if non-nil, zero value otherwise.

### GetWidthOk

`func (o *VehicleCreate) GetWidthOk() (*float32, bool)`

GetWidthOk returns a tuple with the Width field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWidth

`func (o *VehicleCreate) SetWidth(v float32)`

SetWidth sets Width field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


