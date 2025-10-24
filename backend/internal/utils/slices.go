package utils

func Filter[T any](slice []T, predicate func(T) bool) []T {
	result := make([]T, 0)

	for _, item := range slice {
		if predicate(item) {
			result = append(result, item)
		}
	}

	return result
}

func Map[T, K any](slice []T, fn func(T) K) []K {
	result := make([]K, len(slice))

	for i, item := range slice {
		result[i] = fn(item)
	}

	return result
}

func MapIdx[T, K any](slice []T, fn func(T, int) K) []K {
	result := make([]K, len(slice))

	for i, item := range slice {
		result[i] = fn(item, i)
	}

	return result
}

func MapKeysSlice[K comparable, V, T any](m map[K]V, fn func(K, V) T) []T {
	result := make([]T, 0, len(m))

	for k, v := range m {
		result = append(result, fn(k, v))
	}

	return result
}

func Reduce[T, K any](slice []T, fn func(K, T) K, initial K) K {
	result := initial

	for _, item := range slice {
		result = fn(result, item)
	}

	return result
}

func ReduceIdx[T, K any](slice []T, fn func(K, T, int) K, initial K) K {
	result := initial

	for i, item := range slice {
		result = fn(result, item, i)
	}

	return result
}

func Contains[T comparable](slice []T, item T) bool {
	for _, i := range slice {
		if i == item {
			return true
		}
	}

	return false
}
