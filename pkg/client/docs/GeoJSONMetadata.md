# GeoJSONMetadata

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**EndPoint** | [**GeoJSONLocation**](GeoJSONLocation.md) |  | 
**StartPoint** | [**GeoJSONLocation**](GeoJSONLocation.md) |  | 
**WateringPoint** | [**GeoJSONLocation**](GeoJSONLocation.md) |  | 

## Methods

### NewGeoJSONMetadata

`func NewGeoJSONMetadata(endPoint GeoJSONLocation, startPoint GeoJSONLocation, wateringPoint GeoJSONLocation, ) *GeoJSONMetadata`

NewGeoJSONMetadata instantiates a new GeoJSONMetadata object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGeoJSONMetadataWithDefaults

`func NewGeoJSONMetadataWithDefaults() *GeoJSONMetadata`

NewGeoJSONMetadataWithDefaults instantiates a new GeoJSONMetadata object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetEndPoint

`func (o *GeoJSONMetadata) GetEndPoint() GeoJSONLocation`

GetEndPoint returns the EndPoint field if non-nil, zero value otherwise.

### GetEndPointOk

`func (o *GeoJSONMetadata) GetEndPointOk() (*GeoJSONLocation, bool)`

GetEndPointOk returns a tuple with the EndPoint field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEndPoint

`func (o *GeoJSONMetadata) SetEndPoint(v GeoJSONLocation)`

SetEndPoint sets EndPoint field to given value.


### GetStartPoint

`func (o *GeoJSONMetadata) GetStartPoint() GeoJSONLocation`

GetStartPoint returns the StartPoint field if non-nil, zero value otherwise.

### GetStartPointOk

`func (o *GeoJSONMetadata) GetStartPointOk() (*GeoJSONLocation, bool)`

GetStartPointOk returns a tuple with the StartPoint field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStartPoint

`func (o *GeoJSONMetadata) SetStartPoint(v GeoJSONLocation)`

SetStartPoint sets StartPoint field to given value.


### GetWateringPoint

`func (o *GeoJSONMetadata) GetWateringPoint() GeoJSONLocation`

GetWateringPoint returns the WateringPoint field if non-nil, zero value otherwise.

### GetWateringPointOk

`func (o *GeoJSONMetadata) GetWateringPointOk() (*GeoJSONLocation, bool)`

GetWateringPointOk returns a tuple with the WateringPoint field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetWateringPoint

`func (o *GeoJSONMetadata) SetWateringPoint(v GeoJSONLocation)`

SetWateringPoint sets WateringPoint field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


