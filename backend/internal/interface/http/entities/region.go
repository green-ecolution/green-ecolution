package entities

type RegionResponse struct {
	ID   int32  `json:"id"`
	Name string `json:"name"`
} //	@Name	Region

type RegionListResponse struct {
	Data       []*RegionResponse `json:"data"`
	Pagination *Pagination       `json:"pagination,omitempty" validate:"optional"`
} //	@Name	RegionList
