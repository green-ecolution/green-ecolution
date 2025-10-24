# GeoJsonGeometry

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Coordinates** | **[][]float32** |  | 
**Type** | [**GeoJsonType**](GeoJsonType.md) |  | 

## Methods

### NewGeoJsonGeometry

`func NewGeoJsonGeometry(coordinates [][]float32, type_ GeoJsonType, ) *GeoJsonGeometry`

NewGeoJsonGeometry instantiates a new GeoJsonGeometry object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGeoJsonGeometryWithDefaults

`func NewGeoJsonGeometryWithDefaults() *GeoJsonGeometry`

NewGeoJsonGeometryWithDefaults instantiates a new GeoJsonGeometry object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCoordinates

`func (o *GeoJsonGeometry) GetCoordinates() [][]float32`

GetCoordinates returns the Coordinates field if non-nil, zero value otherwise.

### GetCoordinatesOk

`func (o *GeoJsonGeometry) GetCoordinatesOk() (*[][]float32, bool)`

GetCoordinatesOk returns a tuple with the Coordinates field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCoordinates

`func (o *GeoJsonGeometry) SetCoordinates(v [][]float32)`

SetCoordinates sets Coordinates field to given value.


### GetType

`func (o *GeoJsonGeometry) GetType() GeoJsonType`

GetType returns the Type field if non-nil, zero value otherwise.

### GetTypeOk

`func (o *GeoJsonGeometry) GetTypeOk() (*GeoJsonType, bool)`

GetTypeOk returns a tuple with the Type field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetType

`func (o *GeoJsonGeometry) SetType(v GeoJsonType)`

SetType sets Type field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


