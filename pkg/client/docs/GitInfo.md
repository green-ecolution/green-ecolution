# GitInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Branch** | **string** |  | 
**Commit** | **string** |  | 
**Repository** | **string** |  | 

## Methods

### NewGitInfo

`func NewGitInfo(branch string, commit string, repository string, ) *GitInfo`

NewGitInfo instantiates a new GitInfo object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewGitInfoWithDefaults

`func NewGitInfoWithDefaults() *GitInfo`

NewGitInfoWithDefaults instantiates a new GitInfo object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBranch

`func (o *GitInfo) GetBranch() string`

GetBranch returns the Branch field if non-nil, zero value otherwise.

### GetBranchOk

`func (o *GitInfo) GetBranchOk() (*string, bool)`

GetBranchOk returns a tuple with the Branch field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBranch

`func (o *GitInfo) SetBranch(v string)`

SetBranch sets Branch field to given value.


### GetCommit

`func (o *GitInfo) GetCommit() string`

GetCommit returns the Commit field if non-nil, zero value otherwise.

### GetCommitOk

`func (o *GitInfo) GetCommitOk() (*string, bool)`

GetCommitOk returns a tuple with the Commit field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCommit

`func (o *GitInfo) SetCommit(v string)`

SetCommit sets Commit field to given value.


### GetRepository

`func (o *GitInfo) GetRepository() string`

GetRepository returns the Repository field if non-nil, zero value otherwise.

### GetRepositoryOk

`func (o *GitInfo) GetRepositoryOk() (*string, bool)`

GetRepositoryOk returns a tuple with the Repository field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRepository

`func (o *GitInfo) SetRepository(v string)`

SetRepository sets Repository field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


