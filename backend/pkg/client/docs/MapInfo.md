# MapInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Bbox** | **[]float32** |  | 
**Center** | **[]float32** |  | 

## Methods

### NewMapInfo

`func NewMapInfo(bbox []float32, center []float32, ) *MapInfo`

NewMapInfo instantiates a new MapInfo object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewMapInfoWithDefaults

`func NewMapInfoWithDefaults() *MapInfo`

NewMapInfoWithDefaults instantiates a new MapInfo object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBbox

`func (o *MapInfo) GetBbox() []float32`

GetBbox returns the Bbox field if non-nil, zero value otherwise.

### GetBboxOk

`func (o *MapInfo) GetBboxOk() (*[]float32, bool)`

GetBboxOk returns a tuple with the Bbox field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBbox

`func (o *MapInfo) SetBbox(v []float32)`

SetBbox sets Bbox field to given value.


### GetCenter

`func (o *MapInfo) GetCenter() []float32`

GetCenter returns the Center field if non-nil, zero value otherwise.

### GetCenterOk

`func (o *MapInfo) GetCenterOk() (*[]float32, bool)`

GetCenterOk returns a tuple with the Center field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCenter

`func (o *MapInfo) SetCenter(v []float32)`

SetCenter sets Center field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


