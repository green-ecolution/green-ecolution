# WateringPlanUpdate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AdditionalInformation** | **map[string]interface{}** |  | 
**CancellationNote** | **string** |  | 
**Date** | **string** |  | 
**Description** | **string** |  | 
**Evaluation** | [**[]EvaluationValue**](EvaluationValue.md) |  | 
**Provider** | **string** |  | 
**Status** | [**WateringPlanStatus**](WateringPlanStatus.md) |  | 
**TrailerId** | **int32** |  | 
**TransporterId** | **int32** |  | 
**TreeClusterIds** | **[]int32** |  | 
**UserIds** | **[]string** |  | 

## Methods

### NewWateringPlanUpdate

`func NewWateringPlanUpdate(additionalInformation map[string]interface{}, cancellationNote string, date string, description string, evaluation []EvaluationValue, provider string, status WateringPlanStatus, trailerId int32, transporterId int32, treeClusterIds []int32, userIds []string, ) *WateringPlanUpdate`

NewWateringPlanUpdate instantiates a new WateringPlanUpdate object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewWateringPlanUpdateWithDefaults

`func NewWateringPlanUpdateWithDefaults() *WateringPlanUpdate`

NewWateringPlanUpdateWithDefaults instantiates a new WateringPlanUpdate object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAdditionalInformation

`func (o *WateringPlanUpdate) GetAdditionalInformation() map[string]interface{}`

GetAdditionalInformation returns the AdditionalInformation field if non-nil, zero value otherwise.

### GetAdditionalInformationOk

`func (o *WateringPlanUpdate) GetAdditionalInformationOk() (*map[string]interface{}, bool)`

GetAdditionalInformationOk returns a tuple with the AdditionalInformation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAdditionalInformation

`func (o *WateringPlanUpdate) SetAdditionalInformation(v map[string]interface{})`

SetAdditionalInformation sets AdditionalInformation field to given value.


### GetCancellationNote

`func (o *WateringPlanUpdate) GetCancellationNote() string`

GetCancellationNote returns the CancellationNote field if non-nil, zero value otherwise.

### GetCancellationNoteOk

`func (o *WateringPlanUpdate) GetCancellationNoteOk() (*string, bool)`

GetCancellationNoteOk returns a tuple with the CancellationNote field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCancellationNote

`func (o *WateringPlanUpdate) SetCancellationNote(v string)`

SetCancellationNote sets CancellationNote field to given value.


### GetDate

`func (o *WateringPlanUpdate) GetDate() string`

GetDate returns the Date field if non-nil, zero value otherwise.

### GetDateOk

`func (o *WateringPlanUpdate) GetDateOk() (*string, bool)`

GetDateOk returns a tuple with the Date field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDate

`func (o *WateringPlanUpdate) SetDate(v string)`

SetDate sets Date field to given value.


### GetDescription

`func (o *WateringPlanUpdate) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *WateringPlanUpdate) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *WateringPlanUpdate) SetDescription(v string)`

SetDescription sets Description field to given value.


### GetEvaluation

`func (o *WateringPlanUpdate) GetEvaluation() []EvaluationValue`

GetEvaluation returns the Evaluation field if non-nil, zero value otherwise.

### GetEvaluationOk

`func (o *WateringPlanUpdate) GetEvaluationOk() (*[]EvaluationValue, bool)`

GetEvaluationOk returns a tuple with the Evaluation field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluation

`func (o *WateringPlanUpdate) SetEvaluation(v []EvaluationValue)`

SetEvaluation sets Evaluation field to given value.


### GetProvider

`func (o *WateringPlanUpdate) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *WateringPlanUpdate) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *WateringPlanUpdate) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetStatus

`func (o *WateringPlanUpdate) GetStatus() WateringPlanStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *WateringPlanUpdate) GetStatusOk() (*WateringPlanStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *WateringPlanUpdate) SetStatus(v WateringPlanStatus)`

SetStatus sets Status field to given value.


### GetTrailerId

`func (o *WateringPlanUpdate) GetTrailerId() int32`

GetTrailerId returns the TrailerId field if non-nil, zero value otherwise.

### GetTrailerIdOk

`func (o *WateringPlanUpdate) GetTrailerIdOk() (*int32, bool)`

GetTrailerIdOk returns a tuple with the TrailerId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTrailerId

`func (o *WateringPlanUpdate) SetTrailerId(v int32)`

SetTrailerId sets TrailerId field to given value.


### GetTransporterId

`func (o *WateringPlanUpdate) GetTransporterId() int32`

GetTransporterId returns the TransporterId field if non-nil, zero value otherwise.

### GetTransporterIdOk

`func (o *WateringPlanUpdate) GetTransporterIdOk() (*int32, bool)`

GetTransporterIdOk returns a tuple with the TransporterId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTransporterId

`func (o *WateringPlanUpdate) SetTransporterId(v int32)`

SetTransporterId sets TransporterId field to given value.


### GetTreeClusterIds

`func (o *WateringPlanUpdate) GetTreeClusterIds() []int32`

GetTreeClusterIds returns the TreeClusterIds field if non-nil, zero value otherwise.

### GetTreeClusterIdsOk

`func (o *WateringPlanUpdate) GetTreeClusterIdsOk() (*[]int32, bool)`

GetTreeClusterIdsOk returns a tuple with the TreeClusterIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTreeClusterIds

`func (o *WateringPlanUpdate) SetTreeClusterIds(v []int32)`

SetTreeClusterIds sets TreeClusterIds field to given value.


### GetUserIds

`func (o *WateringPlanUpdate) GetUserIds() []string`

GetUserIds returns the UserIds field if non-nil, zero value otherwise.

### GetUserIdsOk

`func (o *WateringPlanUpdate) GetUserIdsOk() (*[]string, bool)`

GetUserIdsOk returns a tuple with the UserIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUserIds

`func (o *WateringPlanUpdate) SetUserIds(v []string)`

SetUserIds sets UserIds field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


