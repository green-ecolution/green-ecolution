# ServiceStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Enabled** | **bool** |  | 
**Healthy** | **bool** |  | 
**LastChecked** | Pointer to **string** |  | [optional] 
**Message** | Pointer to **string** |  | [optional] 
**Name** | **string** |  | 
**ResponseTimeMs** | Pointer to **float32** |  | [optional] 

## Methods

### NewServiceStatus

`func NewServiceStatus(enabled bool, healthy bool, name string, ) *ServiceStatus`

NewServiceStatus instantiates a new ServiceStatus object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewServiceStatusWithDefaults

`func NewServiceStatusWithDefaults() *ServiceStatus`

NewServiceStatusWithDefaults instantiates a new ServiceStatus object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetEnabled

`func (o *ServiceStatus) GetEnabled() bool`

GetEnabled returns the Enabled field if non-nil, zero value otherwise.

### GetEnabledOk

`func (o *ServiceStatus) GetEnabledOk() (*bool, bool)`

GetEnabledOk returns a tuple with the Enabled field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEnabled

`func (o *ServiceStatus) SetEnabled(v bool)`

SetEnabled sets Enabled field to given value.


### GetHealthy

`func (o *ServiceStatus) GetHealthy() bool`

GetHealthy returns the Healthy field if non-nil, zero value otherwise.

### GetHealthyOk

`func (o *ServiceStatus) GetHealthyOk() (*bool, bool)`

GetHealthyOk returns a tuple with the Healthy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHealthy

`func (o *ServiceStatus) SetHealthy(v bool)`

SetHealthy sets Healthy field to given value.


### GetLastChecked

`func (o *ServiceStatus) GetLastChecked() string`

GetLastChecked returns the LastChecked field if non-nil, zero value otherwise.

### GetLastCheckedOk

`func (o *ServiceStatus) GetLastCheckedOk() (*string, bool)`

GetLastCheckedOk returns a tuple with the LastChecked field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLastChecked

`func (o *ServiceStatus) SetLastChecked(v string)`

SetLastChecked sets LastChecked field to given value.

### HasLastChecked

`func (o *ServiceStatus) HasLastChecked() bool`

HasLastChecked returns a boolean if a field has been set.

### GetMessage

`func (o *ServiceStatus) GetMessage() string`

GetMessage returns the Message field if non-nil, zero value otherwise.

### GetMessageOk

`func (o *ServiceStatus) GetMessageOk() (*string, bool)`

GetMessageOk returns a tuple with the Message field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMessage

`func (o *ServiceStatus) SetMessage(v string)`

SetMessage sets Message field to given value.

### HasMessage

`func (o *ServiceStatus) HasMessage() bool`

HasMessage returns a boolean if a field has been set.

### GetName

`func (o *ServiceStatus) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *ServiceStatus) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *ServiceStatus) SetName(v string)`

SetName sets Name field to given value.


### GetResponseTimeMs

`func (o *ServiceStatus) GetResponseTimeMs() float32`

GetResponseTimeMs returns the ResponseTimeMs field if non-nil, zero value otherwise.

### GetResponseTimeMsOk

`func (o *ServiceStatus) GetResponseTimeMsOk() (*float32, bool)`

GetResponseTimeMsOk returns a tuple with the ResponseTimeMs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResponseTimeMs

`func (o *ServiceStatus) SetResponseTimeMs(v float32)`

SetResponseTimeMs sets ResponseTimeMs field to given value.

### HasResponseTimeMs

`func (o *ServiceStatus) HasResponseTimeMs() bool`

HasResponseTimeMs returns a boolean if a field has been set.


[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


