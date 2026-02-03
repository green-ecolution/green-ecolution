# AppInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BuildTime** | **string** |  | 
**Git** | [**GitInfo**](GitInfo.md) |  | 
**GoVersion** | **string** |  | 
**Map** | [**MapInfo**](MapInfo.md) |  | 
**Version** | **string** |  | 
**VersionInfo** | [**VersionInfo**](VersionInfo.md) |  | 

## Methods

### NewAppInfo

`func NewAppInfo(buildTime string, git GitInfo, goVersion string, map_ MapInfo, version string, versionInfo VersionInfo, ) *AppInfo`

NewAppInfo instantiates a new AppInfo object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewAppInfoWithDefaults

`func NewAppInfoWithDefaults() *AppInfo`

NewAppInfoWithDefaults instantiates a new AppInfo object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBuildTime

`func (o *AppInfo) GetBuildTime() string`

GetBuildTime returns the BuildTime field if non-nil, zero value otherwise.

### GetBuildTimeOk

`func (o *AppInfo) GetBuildTimeOk() (*string, bool)`

GetBuildTimeOk returns a tuple with the BuildTime field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBuildTime

`func (o *AppInfo) SetBuildTime(v string)`

SetBuildTime sets BuildTime field to given value.


### GetGit

`func (o *AppInfo) GetGit() GitInfo`

GetGit returns the Git field if non-nil, zero value otherwise.

### GetGitOk

`func (o *AppInfo) GetGitOk() (*GitInfo, bool)`

GetGitOk returns a tuple with the Git field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGit

`func (o *AppInfo) SetGit(v GitInfo)`

SetGit sets Git field to given value.


### GetGoVersion

`func (o *AppInfo) GetGoVersion() string`

GetGoVersion returns the GoVersion field if non-nil, zero value otherwise.

### GetGoVersionOk

`func (o *AppInfo) GetGoVersionOk() (*string, bool)`

GetGoVersionOk returns a tuple with the GoVersion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGoVersion

`func (o *AppInfo) SetGoVersion(v string)`

SetGoVersion sets GoVersion field to given value.


### GetMap

`func (o *AppInfo) GetMap() MapInfo`

GetMap returns the Map field if non-nil, zero value otherwise.

### GetMapOk

`func (o *AppInfo) GetMapOk() (*MapInfo, bool)`

GetMapOk returns a tuple with the Map field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMap

`func (o *AppInfo) SetMap(v MapInfo)`

SetMap sets Map field to given value.


### GetVersion

`func (o *AppInfo) GetVersion() string`

GetVersion returns the Version field if non-nil, zero value otherwise.

### GetVersionOk

`func (o *AppInfo) GetVersionOk() (*string, bool)`

GetVersionOk returns a tuple with the Version field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVersion

`func (o *AppInfo) SetVersion(v string)`

SetVersion sets Version field to given value.


### GetVersionInfo

`func (o *AppInfo) GetVersionInfo() VersionInfo`

GetVersionInfo returns the VersionInfo field if non-nil, zero value otherwise.

### GetVersionInfoOk

`func (o *AppInfo) GetVersionInfoOk() (*VersionInfo, bool)`

GetVersionInfoOk returns a tuple with the VersionInfo field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVersionInfo

`func (o *AppInfo) SetVersionInfo(v VersionInfo)`

SetVersionInfo sets VersionInfo field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


