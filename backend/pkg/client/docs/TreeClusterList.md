# TreeClusterList

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Data** | [**[]TreeClusterInList**](TreeClusterInList.md) |  | 
**Pagination** | Pointer to [**Pagination**](Pagination.md) |  | [optional] 

## Methods

### NewTreeClusterList

`func NewTreeClusterList(data []TreeClusterInList, ) *TreeClusterList`

NewTreeClusterList instantiates a new TreeClusterList object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewTreeClusterListWithDefaults

`func NewTreeClusterListWithDefaults() *TreeClusterList`

NewTreeClusterListWithDefaults instantiates a new TreeClusterList object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetData

`func (o *TreeClusterList) GetData() []TreeClusterInList`

GetData returns the Data field if non-nil, zero value otherwise.

### GetDataOk

`func (o *TreeClusterList) GetDataOk() (*[]TreeClusterInList, bool)`

GetDataOk returns a tuple with the Data field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetData

`func (o *TreeClusterList) SetData(v []TreeClusterInList)`

SetData sets Data field to given value.


### GetPagination

`func (o *TreeClusterList) GetPagination() Pagination`

GetPagination returns the Pagination field if non-nil, zero value otherwise.

### GetPaginationOk

`func (o *TreeClusterList) GetPaginationOk() (*Pagination, bool)`

GetPaginationOk returns a tuple with the Pagination field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPagination

`func (o *TreeClusterList) SetPagination(v Pagination)`

SetPagination sets Pagination field to given value.

### HasPagination

`func (o *TreeClusterList) HasPagination() bool`

HasPagination returns a boolean if a field has been set.


[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


