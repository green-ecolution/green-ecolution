# GeoJsonFeature

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Bbox** | **[]float32** |  | 
**Geometry** | [**GeoJsonGeometry**](GeoJsonGeometry.md) |  | 
**Properties** | **map[string]map[string]interface{}** |  | 
**Type** | [**GeoJsonType**](GeoJsonType.md) |  | 

## Methods

### NewGeoJsonFeature

`func NewGeoJsonFeature(bbox []float32, geometry GeoJsonGeometry, properties map[string]map[string]interface{}, type_ GeoJsonType, ) *GeoJsonFeature`

NewGeoJsonFeature instantiates a new GeoJsonFeature object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGeoJsonFeatureWithDefaults

`func NewGeoJsonFeatureWithDefaults() *GeoJsonFeature`

NewGeoJsonFeatureWithDefaults instantiates a new GeoJsonFeature object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBbox

`func (o *GeoJsonFeature) GetBbox() []float32`

GetBbox returns the Bbox field if non-nil, zero value otherwise.

### GetBboxOk

`func (o *GeoJsonFeature) GetBboxOk() (*[]float32, bool)`

GetBboxOk returns a tuple with the Bbox field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBbox

`func (o *GeoJsonFeature) SetBbox(v []float32)`

SetBbox sets Bbox field to given value.


### GetGeometry

`func (o *GeoJsonFeature) GetGeometry() GeoJsonGeometry`

GetGeometry returns the Geometry field if non-nil, zero value otherwise.

### GetGeometryOk

`func (o *GeoJsonFeature) GetGeometryOk() (*GeoJsonGeometry, bool)`

GetGeometryOk returns a tuple with the Geometry field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGeometry

`func (o *GeoJsonFeature) SetGeometry(v GeoJsonGeometry)`

SetGeometry sets Geometry field to given value.


### GetProperties

`func (o *GeoJsonFeature) GetProperties() map[string]map[string]interface{}`

GetProperties returns the Properties field if non-nil, zero value otherwise.

### GetPropertiesOk

`func (o *GeoJsonFeature) GetPropertiesOk() (*map[string]map[string]interface{}, bool)`

GetPropertiesOk returns a tuple with the Properties field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProperties

`func (o *GeoJsonFeature) SetProperties(v map[string]map[string]interface{})`

SetProperties sets Properties field to given value.


### GetType

`func (o *GeoJsonFeature) GetType() GeoJsonType`

GetType returns the Type field if non-nil, zero value otherwise.

### GetTypeOk

`func (o *GeoJsonFeature) GetTypeOk() (*GeoJsonType, bool)`

GetTypeOk returns a tuple with the Type field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetType

`func (o *GeoJsonFeature) SetType(v GeoJsonType)`

SetType sets Type field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


