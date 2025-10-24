package entities

type Pagination struct {
	Total       int64  `json:"total_records"`
	CurrentPage int32  `json:"current_page"`
	TotalPages  int32  `json:"total_pages"`
	NextPage    *int32 `json:"next_page"`
	PrevPage    *int32 `json:"prev_page"`
} //	@Name	Pagination
