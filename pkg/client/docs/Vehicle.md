# Vehicle

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**ArchivedAt** | **string** |  | 
**CreatedAt** | **string** |  | 
**Description** | **string** |  | 
**DrivingLicense** | [**DrivingLicense**](DrivingLicense.md) |  | 
**Height** | **float32** |  | 
**Id** | **int32** |  | 
**Length** | **float32** |  | 
**Model** | **string** |  | 
**NumberPlate** | **string** |  | 
**Provider** | **string** |  | 
**Status** | [**VehicleStatus**](VehicleStatus.md) |  | 
**Type** | [**VehicleType**](VehicleType.md) |  | 
**UpdatedAt** | **string** |  | 
**WaterCapacity** | **float32** |  | 
**Weight** | **float32** |  | 
**Width** | **float32** |  | 

## Methods

### NewVehicle

`func NewVehicle(archivedAt string, createdAt string, description string, drivingLicense DrivingLicense, height float32, id int32, length float32, model string, numberPlate string, provider string, status VehicleStatus, type_ VehicleType, updatedAt string, waterCapacity float32, weight float32, width float32, ) *Vehicle`

NewVehicle instantiates a new Vehicle object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewVehicleWithDefaults

`func NewVehicleWithDefaults() *Vehicle`

NewVehicleWithDefaults instantiates a new Vehicle object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *Vehicle) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *Vehicle) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *Vehicle) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *Vehicle) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetArchivedAt

`func (o *Vehicle) GetArchivedAt() string`

GetArchivedAt returns the ArchivedAt field if non-nil, zero value otherwise.

### GetArchivedAtOk

`func (o *Vehicle) GetArchivedAtOk() (*string, bool)`

GetArchivedAtOk returns a tuple with the ArchivedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetArchivedAt

`func (o *Vehicle) SetArchivedAt(v string)`

SetArchivedAt sets ArchivedAt field to given value.


### GetCreatedAt

`func (o *Vehicle) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *Vehicle) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *Vehicle) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetDescription

`func (o *Vehicle) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *Vehicle) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *Vehicle) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetDrivingLicense

`func (o *Vehicle) GetDrivingLicense() DrivingLicense`

GetDrivingLicense returns the DrivingLicense field if non-nil, zero value otherwise.

### GetDrivingLicenseOk

`func (o *Vehicle) GetDrivingLicenseOk() (*DrivingLicense, bool)`

GetDrivingLicenseOk returns a tuple with the DrivingLicense field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDrivingLicense

`func (o *Vehicle) SetDrivingLicense(v DrivingLicense)`

SetDrivingLicense sets DrivingLicense field to given value.


### GetHeight

`func (o *Vehicle) GetHeight() float32`

GetHeight returns the Height field if non-nil, zero value otherwise.

### GetHeightOk

`func (o *Vehicle) GetHeightOk() (*float32, bool)`

GetHeightOk returns a tuple with the Height field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHeight

`func (o *Vehicle) SetHeight(v float32)`

SetHeight sets Height field to given value.


### GetId

`func (o *Vehicle) GetId() int32`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *Vehicle) GetIdOk() (*int32, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *Vehicle) SetId(v int32)`

SetId sets Id field to given value.


### GetLength

`func (o *Vehicle) GetLength() float32`

GetLength returns the Length field if non-nil, zero value otherwise.

### GetLengthOk

`func (o *Vehicle) GetLengthOk() (*float32, bool)`

GetLengthOk returns a tuple with the Length field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLength

`func (o *Vehicle) SetLength(v float32)`

SetLength sets Length field to given value.


### GetModel

`func (o *Vehicle) GetModel() string`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *Vehicle) GetModelOk() (*string, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *Vehicle) SetModel(v string)`

SetModel sets Model field to given value.


### GetNumberPlate

`func (o *Vehicle) GetNumberPlate() string`

GetNumberPlate returns the NumberPlate field if non-nil, zero value otherwise.

### GetNumberPlateOk

`func (o *Vehicle) GetNumberPlateOk() (*string, bool)`

GetNumberPlateOk returns a tuple with the NumberPlate field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNumberPlate

`func (o *Vehicle) SetNumberPlate(v string)`

SetNumberPlate sets NumberPlate field to given value.


### GetProvider

`func (o *Vehicle) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *Vehicle) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *Vehicle) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetStatus

`func (o *Vehicle) GetStatus() VehicleStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *Vehicle) GetStatusOk() (*VehicleStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *Vehicle) SetStatus(v VehicleStatus)`

SetStatus sets Status field to given value.


### GetType

`func (o *Vehicle) GetType() VehicleType`

GetType returns the Type field if non-nil, zero value otherwise.

### GetTypeOk

`func (o *Vehicle) GetTypeOk() (*VehicleType, bool)`

GetTypeOk returns a tuple with the Type field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetType

`func (o *Vehicle) SetType(v VehicleType)`

SetType sets Type field to given value.


### GetUpdatedAt

`func (o *Vehicle) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *Vehicle) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *Vehicle) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.


### GetWaterCapacity

`func (o *Vehicle) GetWaterCapacity() float32`

GetWaterCapacity returns the WaterCapacity field if non-nil, zero value otherwise.

### GetWaterCapacityOk

`func (o *Vehicle) GetWaterCapacityOk() (*float32, bool)`

GetWaterCapacityOk returns a tuple with the WaterCapacity field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWaterCapacity

`func (o *Vehicle) SetWaterCapacity(v float32)`

SetWaterCapacity sets WaterCapacity field to given value.


### GetWeight

`func (o *Vehicle) GetWeight() float32`

GetWeight returns the Weight field if non-nil, zero value otherwise.

### GetWeightOk

`func (o *Vehicle) GetWeightOk() (*float32, bool)`

GetWeightOk returns a tuple with the Weight field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWeight

`func (o *Vehicle) SetWeight(v float32)`

SetWeight sets Weight field to given value.


### GetWidth

`func (o *Vehicle) GetWidth() float32`

GetWidth returns the Width field if non-nil, zero value otherwise.

### GetWidthOk

`func (o *Vehicle) GetWidthOk() (*float32, bool)`

GetWidthOk returns a tuple with the Width field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWidth

`func (o *Vehicle) SetWidth(v float32)`

SetWidth sets Width field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


