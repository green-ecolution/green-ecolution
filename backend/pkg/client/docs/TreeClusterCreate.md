# TreeClusterCreate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**Address** | **string** |  | 
**Description** | **string** |  | 
**Name** | **string** |  | 
**Provider** | Pointer to **string** |  | [optional] 
**SoilCondition** | [**SoilCondition**](SoilCondition.md) |  | 
**TreeIds** | **[]int32** |  | 

## Methods

### NewTreeClusterCreate

`func NewTreeClusterCreate(address string, description string, name string, soilCondition SoilCondition, treeIds []int32, ) *TreeClusterCreate`

NewTreeClusterCreate instantiates a new TreeClusterCreate object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTreeClusterCreateWithDefaults

`func NewTreeClusterCreateWithDefaults() *TreeClusterCreate`

NewTreeClusterCreateWithDefaults instantiates a new TreeClusterCreate object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *TreeClusterCreate) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *TreeClusterCreate) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *TreeClusterCreate) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *TreeClusterCreate) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetAddress

`func (o *TreeClusterCreate) GetAddress() string`

GetAddress returns the Address field if non-nil, zero value otherwise.

### GetAddressOk

`func (o *TreeClusterCreate) GetAddressOk() (*string, bool)`

GetAddressOk returns a tuple with the Address field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAddress

`func (o *TreeClusterCreate) SetAddress(v string)`

SetAddress sets Address field to given value.


### GetDescription

`func (o *TreeClusterCreate) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *TreeClusterCreate) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *TreeClusterCreate) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetName

`func (o *TreeClusterCreate) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *TreeClusterCreate) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *TreeClusterCreate) SetName(v string)`

SetName sets Name field to given value.


### GetProvider

`func (o *TreeClusterCreate) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *TreeClusterCreate) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *TreeClusterCreate) SetProvider(v string)`

SetProvider sets Provider field to given value.

### HasProvider

`func (o *TreeClusterCreate) HasProvider() bool`

HasProvider returns a boolean if a field has been set.

### GetSoilCondition

`func (o *TreeClusterCreate) GetSoilCondition() SoilCondition`

GetSoilCondition returns the SoilCondition field if non-nil, zero value otherwise.

### GetSoilConditionOk

`func (o *TreeClusterCreate) GetSoilConditionOk() (*SoilCondition, bool)`

GetSoilConditionOk returns a tuple with the SoilCondition field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSoilCondition

`func (o *TreeClusterCreate) SetSoilCondition(v SoilCondition)`

SetSoilCondition sets SoilCondition field to given value.


### GetTreeIds

`func (o *TreeClusterCreate) GetTreeIds() []int32`

GetTreeIds returns the TreeIds field if non-nil, zero value otherwise.

### GetTreeIdsOk

`func (o *TreeClusterCreate) GetTreeIdsOk() (*[]int32, bool)`

GetTreeIdsOk returns a tuple with the TreeIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeIds

`func (o *TreeClusterCreate) SetTreeIds(v []int32)`

SetTreeIds sets TreeIds field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


