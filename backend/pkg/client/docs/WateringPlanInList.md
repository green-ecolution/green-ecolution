# WateringPlanInList

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**CancellationNote** | **string** |  | 
**CreatedAt** | **string** |  | 
**Date** | **string** |  | 
**Description** | **string** |  | 
**Distance** | **float32** |  | 
**Id** | **int32** |  | 
**Provider** | Pointer to **string** |  | [optional] 
**Status** | [**WateringPlanStatus**](WateringPlanStatus.md) |  | 
**TotalWaterRequired** | **float32** |  | 
**Trailer** | Pointer to [**Vehicle**](Vehicle.md) |  | [optional] 
**Transporter** | [**Vehicle**](Vehicle.md) |  | 
**Treeclusters** | [**[]TreeClusterInList**](TreeClusterInList.md) |  | 
**UpdatedAt** | **string** |  | 
**UserIds** | **[]string** |  | 

## Methods

### NewWateringPlanInList

`func NewWateringPlanInList(cancellationNote string, createdAt string, date string, description string, distance float32, id int32, status WateringPlanStatus, totalWaterRequired float32, transporter Vehicle, treeclusters []TreeClusterInList, updatedAt string, userIds []string, ) *WateringPlanInList`

NewWateringPlanInList instantiates a new WateringPlanInList object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewWateringPlanInListWithDefaults

`func NewWateringPlanInListWithDefaults() *WateringPlanInList`

NewWateringPlanInListWithDefaults instantiates a new WateringPlanInList object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *WateringPlanInList) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *WateringPlanInList) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *WateringPlanInList) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *WateringPlanInList) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetCancellationNote

`func (o *WateringPlanInList) GetCancellationNote() string`

GetCancellationNote returns the CancellationNote field if non-nil, zero value otherwise.

### GetCancellationNoteOk

`func (o *WateringPlanInList) GetCancellationNoteOk() (*string, bool)`

GetCancellationNoteOk returns a tuple with the CancellationNote field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCancellationNote

`func (o *WateringPlanInList) SetCancellationNote(v string)`

SetCancellationNote sets CancellationNote field to given value.


### GetCreatedAt

`func (o *WateringPlanInList) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *WateringPlanInList) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *WateringPlanInList) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetDate

`func (o *WateringPlanInList) GetDate() string`

GetDate returns the Date field if non-nil, zero value otherwise.

### GetDateOk

`func (o *WateringPlanInList) GetDateOk() (*string, bool)`

GetDateOk returns a tuple with the Date field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDate

`func (o *WateringPlanInList) SetDate(v string)`

SetDate sets Date field to given value.


### GetDescription

`func (o *WateringPlanInList) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *WateringPlanInList) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *WateringPlanInList) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetDistance

`func (o *WateringPlanInList) GetDistance() float32`

GetDistance returns the Distance field if non-nil, zero value otherwise.

### GetDistanceOk

`func (o *WateringPlanInList) GetDistanceOk() (*float32, bool)`

GetDistanceOk returns a tuple with the Distance field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDistance

`func (o *WateringPlanInList) SetDistance(v float32)`

SetDistance sets Distance field to given value.


### GetId

`func (o *WateringPlanInList) GetId() int32`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *WateringPlanInList) GetIdOk() (*int32, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *WateringPlanInList) SetId(v int32)`

SetId sets Id field to given value.


### GetProvider

`func (o *WateringPlanInList) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *WateringPlanInList) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *WateringPlanInList) SetProvider(v string)`

SetProvider sets Provider field to given value.

### HasProvider

`func (o *WateringPlanInList) HasProvider() bool`

HasProvider returns a boolean if a field has been set.

### GetStatus

`func (o *WateringPlanInList) GetStatus() WateringPlanStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *WateringPlanInList) GetStatusOk() (*WateringPlanStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *WateringPlanInList) SetStatus(v WateringPlanStatus)`

SetStatus sets Status field to given value.


### GetTotalWaterRequired

`func (o *WateringPlanInList) GetTotalWaterRequired() float32`

GetTotalWaterRequired returns the TotalWaterRequired field if non-nil, zero value otherwise.

### GetTotalWaterRequiredOk

`func (o *WateringPlanInList) GetTotalWaterRequiredOk() (*float32, bool)`

GetTotalWaterRequiredOk returns a tuple with the TotalWaterRequired field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTotalWaterRequired

`func (o *WateringPlanInList) SetTotalWaterRequired(v float32)`

SetTotalWaterRequired sets TotalWaterRequired field to given value.


### GetTrailer

`func (o *WateringPlanInList) GetTrailer() Vehicle`

GetTrailer returns the Trailer field if non-nil, zero value otherwise.

### GetTrailerOk

`func (o *WateringPlanInList) GetTrailerOk() (*Vehicle, bool)`

GetTrailerOk returns a tuple with the Trailer field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrailer

`func (o *WateringPlanInList) SetTrailer(v Vehicle)`

SetTrailer sets Trailer field to given value.

### HasTrailer

`func (o *WateringPlanInList) HasTrailer() bool`

HasTrailer returns a boolean if a field has been set.

### GetTransporter

`func (o *WateringPlanInList) GetTransporter() Vehicle`

GetTransporter returns the Transporter field if non-nil, zero value otherwise.

### GetTransporterOk

`func (o *WateringPlanInList) GetTransporterOk() (*Vehicle, bool)`

GetTransporterOk returns a tuple with the Transporter field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTransporter

`func (o *WateringPlanInList) SetTransporter(v Vehicle)`

SetTransporter sets Transporter field to given value.


### GetTreeclusters

`func (o *WateringPlanInList) GetTreeclusters() []TreeClusterInList`

GetTreeclusters returns the Treeclusters field if non-nil, zero value otherwise.

### GetTreeclustersOk

`func (o *WateringPlanInList) GetTreeclustersOk() (*[]TreeClusterInList, bool)`

GetTreeclustersOk returns a tuple with the Treeclusters field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeclusters

`func (o *WateringPlanInList) SetTreeclusters(v []TreeClusterInList)`

SetTreeclusters sets Treeclusters field to given value.


### GetUpdatedAt

`func (o *WateringPlanInList) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *WateringPlanInList) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *WateringPlanInList) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.


### GetUserIds

`func (o *WateringPlanInList) GetUserIds() []string`

GetUserIds returns the UserIds field if non-nil, zero value otherwise.

### GetUserIdsOk

`func (o *WateringPlanInList) GetUserIdsOk() (*[]string, bool)`

GetUserIdsOk returns a tuple with the UserIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUserIds

`func (o *WateringPlanInList) SetUserIds(v []string)`

SetUserIds sets UserIds field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


