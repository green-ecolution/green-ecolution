package utils

import (
	"iter"
)

func NumberSequence(start int) iter.Seq[int] {
	return func(yield func(int) bool) {
		for i := start; yield(i); i++ {
		}
	}
}
