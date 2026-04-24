package vroom

import "github.com/green-ecolution/green-ecolution/backend/internal/utils"

type VroomType string

const (
	VroomPickup VroomType = "pickup"
)

// Reduce multiple pickups to one
// "start" -> "pickup" -> "pickup" -> "delivery" => "start" -> "pickup" -> "delivery"
//
//nolint:gocritic // ignored because this has to be called in a callback function
func ReduceSteps(acc []*VroomRouteStep, current VroomRouteStep) []*VroomRouteStep {
	if len(acc) == 0 {
		return append(acc, &current)
	}

	prev := acc[len(acc)-1]
	if prev.Type != string(VroomPickup) {
		return append(acc, &current)
	}

	if current.Type != string(VroomPickup) {
		return append(acc, &current)
	}

	prev.Load = current.Load
	return acc
}

func RefillCount(steps []*VroomRouteStep) int {
	return len(utils.Filter(steps, func(step *VroomRouteStep) bool {
		return step.Type == string(VroomPickup)
	}))
}
