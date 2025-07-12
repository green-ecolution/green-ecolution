# GeoJson

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Bbox** | **[]float32** |  | 
**Features** | [**[]GeoJsonFeature**](GeoJsonFeature.md) |  | 
**Metadata** | [**GeoJSONMetadata**](GeoJSONMetadata.md) |  | 
**Type** | [**GeoJsonType**](GeoJsonType.md) |  | 

## Methods

### NewGeoJson

`func NewGeoJson(bbox []float32, features []GeoJsonFeature, metadata GeoJSONMetadata, type_ GeoJsonType, ) *GeoJson`

NewGeoJson instantiates a new GeoJson object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGeoJsonWithDefaults

`func NewGeoJsonWithDefaults() *GeoJson`

NewGeoJsonWithDefaults instantiates a new GeoJson object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBbox

`func (o *GeoJson) GetBbox() []float32`

GetBbox returns the Bbox field if non-nil, zero value otherwise.

### GetBboxOk

`func (o *GeoJson) GetBboxOk() (*[]float32, bool)`

GetBboxOk returns a tuple with the Bbox field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBbox

`func (o *GeoJson) SetBbox(v []float32)`

SetBbox sets Bbox field to given value.


### GetFeatures

`func (o *GeoJson) GetFeatures() []GeoJsonFeature`

GetFeatures returns the Features field if non-nil, zero value otherwise.

### GetFeaturesOk

`func (o *GeoJson) GetFeaturesOk() (*[]GeoJsonFeature, bool)`

GetFeaturesOk returns a tuple with the Features field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFeatures

`func (o *GeoJson) SetFeatures(v []GeoJsonFeature)`

SetFeatures sets Features field to given value.


### GetMetadata

`func (o *GeoJson) GetMetadata() GeoJSONMetadata`

GetMetadata returns the Metadata field if non-nil, zero value otherwise.

### GetMetadataOk

`func (o *GeoJson) GetMetadataOk() (*GeoJSONMetadata, bool)`

GetMetadataOk returns a tuple with the Metadata field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMetadata

`func (o *GeoJson) SetMetadata(v GeoJSONMetadata)`

SetMetadata sets Metadata field to given value.


### GetType

`func (o *GeoJson) GetType() GeoJsonType`

GetType returns the Type field if non-nil, zero value otherwise.

### GetTypeOk

`func (o *GeoJson) GetTypeOk() (*GeoJsonType, bool)`

GetTypeOk returns a tuple with the Type field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetType

`func (o *GeoJson) SetType(v GeoJsonType)`

SetType sets Type field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


