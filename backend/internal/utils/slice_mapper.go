package utils

// MapSlice applies fn to each element of src. Returns nil if src is nil.
func MapSlice[S, D any](src []S, fn func(S) D) []D {
	if src == nil {
		return nil
	}
	result := make([]D, len(src))
	for i, v := range src {
		result[i] = fn(v)
	}
	return result
}

// MapSliceErr applies fn to each element of src, returning early on error. Returns nil if src is nil.
func MapSliceErr[S, D any](src []S, fn func(S) (D, error)) ([]D, error) {
	if src == nil {
		return nil, nil
	}
	result := make([]D, len(src))
	for i, v := range src {
		var err error
		result[i], err = fn(v)
		if err != nil {
			return nil, err
		}
	}
	return result, nil
}
