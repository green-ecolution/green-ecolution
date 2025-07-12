# RouteRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ClusterIds** | **[]int32** |  | 
**TrailerId** | Pointer to **int32** |  | [optional] 
**TransporterId** | **int32** |  | 

## Methods

### NewRouteRequest

`func NewRouteRequest(clusterIds []int32, transporterId int32, ) *RouteRequest`

NewRouteRequest instantiates a new RouteRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewRouteRequestWithDefaults

`func NewRouteRequestWithDefaults() *RouteRequest`

NewRouteRequestWithDefaults instantiates a new RouteRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetClusterIds

`func (o *RouteRequest) GetClusterIds() []int32`

GetClusterIds returns the ClusterIds field if non-nil, zero value otherwise.

### GetClusterIdsOk

`func (o *RouteRequest) GetClusterIdsOk() (*[]int32, bool)`

GetClusterIdsOk returns a tuple with the ClusterIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetClusterIds

`func (o *RouteRequest) SetClusterIds(v []int32)`

SetClusterIds sets ClusterIds field to given value.


### GetTrailerId

`func (o *RouteRequest) GetTrailerId() int32`

GetTrailerId returns the TrailerId field if non-nil, zero value otherwise.

### GetTrailerIdOk

`func (o *RouteRequest) GetTrailerIdOk() (*int32, bool)`

GetTrailerIdOk returns a tuple with the TrailerId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrailerId

`func (o *RouteRequest) SetTrailerId(v int32)`

SetTrailerId sets TrailerId field to given value.

### HasTrailerId

`func (o *RouteRequest) HasTrailerId() bool`

HasTrailerId returns a boolean if a field has been set.

### GetTransporterId

`func (o *RouteRequest) GetTransporterId() int32`

GetTransporterId returns the TransporterId field if non-nil, zero value otherwise.

### GetTransporterIdOk

`func (o *RouteRequest) GetTransporterIdOk() (*int32, bool)`

GetTransporterIdOk returns a tuple with the TransporterId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTransporterId

`func (o *RouteRequest) SetTransporterId(v int32)`

SetTransporterId sets TransporterId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


