# WateringPlan

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | Pointer to **map[string]interface{}** |  | [optional] 
**CancellationNote** | **string** |  | 
**CreatedAt** | **string** |  | 
**Date** | **string** |  | 
**Description** | **string** |  | 
**Distance** | **float32** |  | 
**Duration** | **float32** |  | 
**Evaluation** | [**[]EvaluationValue**](EvaluationValue.md) |  | 
**GpxUrl** | **string** |  | 
**Id** | **int32** |  | 
**Provider** | Pointer to **string** |  | [optional] 
**RefillCount** | **int32** |  | 
**Status** | [**WateringPlanStatus**](WateringPlanStatus.md) |  | 
**TotalWaterRequired** | **float32** |  | 
**Trailer** | Pointer to [**Vehicle**](Vehicle.md) |  | [optional] 
**Transporter** | [**Vehicle**](Vehicle.md) |  | 
**Treeclusters** | [**[]TreeClusterInList**](TreeClusterInList.md) |  | 
**UpdatedAt** | **string** |  | 
**UserIds** | **[]string** |  | 

## Methods

### NewWateringPlan

`func NewWateringPlan(cancellationNote string, createdAt string, date string, description string, distance float32, duration float32, evaluation []EvaluationValue, gpxUrl string, id int32, refillCount int32, status WateringPlanStatus, totalWaterRequired float32, transporter Vehicle, treeclusters []TreeClusterInList, updatedAt string, userIds []string, ) *WateringPlan`

NewWateringPlan instantiates a new WateringPlan object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewWateringPlanWithDefaults

`func NewWateringPlanWithDefaults() *WateringPlan`

NewWateringPlanWithDefaults instantiates a new WateringPlan object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *WateringPlan) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *WateringPlan) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *WateringPlan) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.

### HasAdditionalInformation

`func (o *WateringPlan) HasAdditionalInformation() bool`

HasAdditionalInformation returns a boolean if a field has been set.

### GetCancellationNote

`func (o *WateringPlan) GetCancellationNote() string`

GetCancellationNote returns the CancellationNote field if non-nil, zero value otherwise.

### GetCancellationNoteOk

`func (o *WateringPlan) GetCancellationNoteOk() (*string, bool)`

GetCancellationNoteOk returns a tuple with the CancellationNote field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCancellationNote

`func (o *WateringPlan) SetCancellationNote(v string)`

SetCancellationNote sets CancellationNote field to given value.


### GetCreatedAt

`func (o *WateringPlan) GetCreatedAt() string`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *WateringPlan) GetCreatedAtOk() (*string, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *WateringPlan) SetCreatedAt(v string)`

SetCreatedAt sets CreatedAt field to given value.


### GetDate

`func (o *WateringPlan) GetDate() string`

GetDate returns the Date field if non-nil, zero value otherwise.

### GetDateOk

`func (o *WateringPlan) GetDateOk() (*string, bool)`

GetDateOk returns a tuple with the Date field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDate

`func (o *WateringPlan) SetDate(v string)`

SetDate sets Date field to given value.


### GetDescription

`func (o *WateringPlan) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *WateringPlan) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *WateringPlan) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetDistance

`func (o *WateringPlan) GetDistance() float32`

GetDistance returns the Distance field if non-nil, zero value otherwise.

### GetDistanceOk

`func (o *WateringPlan) GetDistanceOk() (*float32, bool)`

GetDistanceOk returns a tuple with the Distance field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDistance

`func (o *WateringPlan) SetDistance(v float32)`

SetDistance sets Distance field to given value.


### GetDuration

`func (o *WateringPlan) GetDuration() float32`

GetDuration returns the Duration field if non-nil, zero value otherwise.

### GetDurationOk

`func (o *WateringPlan) GetDurationOk() (*float32, bool)`

GetDurationOk returns a tuple with the Duration field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDuration

`func (o *WateringPlan) SetDuration(v float32)`

SetDuration sets Duration field to given value.


### GetEvaluation

`func (o *WateringPlan) GetEvaluation() []EvaluationValue`

GetEvaluation returns the Evaluation field if non-nil, zero value otherwise.

### GetEvaluationOk

`func (o *WateringPlan) GetEvaluationOk() (*[]EvaluationValue, bool)`

GetEvaluationOk returns a tuple with the Evaluation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluation

`func (o *WateringPlan) SetEvaluation(v []EvaluationValue)`

SetEvaluation sets Evaluation field to given value.


### GetGpxUrl

`func (o *WateringPlan) GetGpxUrl() string`

GetGpxUrl returns the GpxUrl field if non-nil, zero value otherwise.

### GetGpxUrlOk

`func (o *WateringPlan) GetGpxUrlOk() (*string, bool)`

GetGpxUrlOk returns a tuple with the GpxUrl field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGpxUrl

`func (o *WateringPlan) SetGpxUrl(v string)`

SetGpxUrl sets GpxUrl field to given value.


### GetId

`func (o *WateringPlan) GetId() int32`

GetId returns the Id field if non-nil, zero value otherwise.

### GetIdOk

`func (o *WateringPlan) GetIdOk() (*int32, bool)`

GetIdOk returns a tuple with the Id field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetId

`func (o *WateringPlan) SetId(v int32)`

SetId sets Id field to given value.


### GetProvider

`func (o *WateringPlan) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *WateringPlan) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *WateringPlan) SetProvider(v string)`

SetProvider sets Provider field to given value.

### HasProvider

`func (o *WateringPlan) HasProvider() bool`

HasProvider returns a boolean if a field has been set.

### GetRefillCount

`func (o *WateringPlan) GetRefillCount() int32`

GetRefillCount returns the RefillCount field if non-nil, zero value otherwise.

### GetRefillCountOk

`func (o *WateringPlan) GetRefillCountOk() (*int32, bool)`

GetRefillCountOk returns a tuple with the RefillCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRefillCount

`func (o *WateringPlan) SetRefillCount(v int32)`

SetRefillCount sets RefillCount field to given value.


### GetStatus

`func (o *WateringPlan) GetStatus() WateringPlanStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *WateringPlan) GetStatusOk() (*WateringPlanStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *WateringPlan) SetStatus(v WateringPlanStatus)`

SetStatus sets Status field to given value.


### GetTotalWaterRequired

`func (o *WateringPlan) GetTotalWaterRequired() float32`

GetTotalWaterRequired returns the TotalWaterRequired field if non-nil, zero value otherwise.

### GetTotalWaterRequiredOk

`func (o *WateringPlan) GetTotalWaterRequiredOk() (*float32, bool)`

GetTotalWaterRequiredOk returns a tuple with the TotalWaterRequired field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTotalWaterRequired

`func (o *WateringPlan) SetTotalWaterRequired(v float32)`

SetTotalWaterRequired sets TotalWaterRequired field to given value.


### GetTrailer

`func (o *WateringPlan) GetTrailer() Vehicle`

GetTrailer returns the Trailer field if non-nil, zero value otherwise.

### GetTrailerOk

`func (o *WateringPlan) GetTrailerOk() (*Vehicle, bool)`

GetTrailerOk returns a tuple with the Trailer field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrailer

`func (o *WateringPlan) SetTrailer(v Vehicle)`

SetTrailer sets Trailer field to given value.

### HasTrailer

`func (o *WateringPlan) HasTrailer() bool`

HasTrailer returns a boolean if a field has been set.

### GetTransporter

`func (o *WateringPlan) GetTransporter() Vehicle`

GetTransporter returns the Transporter field if non-nil, zero value otherwise.

### GetTransporterOk

`func (o *WateringPlan) GetTransporterOk() (*Vehicle, bool)`

GetTransporterOk returns a tuple with the Transporter field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTransporter

`func (o *WateringPlan) SetTransporter(v Vehicle)`

SetTransporter sets Transporter field to given value.


### GetTreeclusters

`func (o *WateringPlan) GetTreeclusters() []TreeClusterInList`

GetTreeclusters returns the Treeclusters field if non-nil, zero value otherwise.

### GetTreeclustersOk

`func (o *WateringPlan) GetTreeclustersOk() (*[]TreeClusterInList, bool)`

GetTreeclustersOk returns a tuple with the Treeclusters field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeclusters

`func (o *WateringPlan) SetTreeclusters(v []TreeClusterInList)`

SetTreeclusters sets Treeclusters field to given value.


### GetUpdatedAt

`func (o *WateringPlan) GetUpdatedAt() string`

GetUpdatedAt returns the UpdatedAt field if non-nil, zero value otherwise.

### GetUpdatedAtOk

`func (o *WateringPlan) GetUpdatedAtOk() (*string, bool)`

GetUpdatedAtOk returns a tuple with the UpdatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUpdatedAt

`func (o *WateringPlan) SetUpdatedAt(v string)`

SetUpdatedAt sets UpdatedAt field to given value.


### GetUserIds

`func (o *WateringPlan) GetUserIds() []string`

GetUserIds returns the UserIds field if non-nil, zero value otherwise.

### GetUserIdsOk

`func (o *WateringPlan) GetUserIdsOk() (*[]string, bool)`

GetUserIdsOk returns a tuple with the UserIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUserIds

`func (o *WateringPlan) SetUserIds(v []string)`

SetUserIds sets UserIds field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


