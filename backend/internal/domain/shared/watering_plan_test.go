package entities

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestWateringPlan_CalculateRequiredWater(t *testing.T) {
	wp := &WateringPlan{
		TreeClusters: []*TreeCluster{
			{Trees: make([]*Tree, 5)},
			{Trees: make([]*Tree, 3)},
		},
	}
	assert.Equal(t, 640.0, wp.CalculateRequiredWater()) // (5+3) * 80

	empty := &WateringPlan{}
	assert.Equal(t, 0.0, empty.CalculateRequiredWater())
}

func TestWateringPlan_ShouldRegenerateRoute(t *testing.T) {
	tc1 := &TreeCluster{ID: 1}
	tc2 := &TreeCluster{ID: 2}
	v1 := &Vehicle{ID: 1}
	v2 := &Vehicle{ID: 2}

	t.Run("cluster count changed", func(t *testing.T) {
		prev := &WateringPlan{TreeClusters: []*TreeCluster{tc1}, Transporter: v1}
		next := &WateringPlan{TreeClusters: []*TreeCluster{tc1, tc2}, Transporter: v1}
		assert.True(t, next.ShouldRegenerateRoute(prev))
	})

	t.Run("transporter changed", func(t *testing.T) {
		prev := &WateringPlan{TreeClusters: []*TreeCluster{tc1}, Transporter: v1}
		next := &WateringPlan{TreeClusters: []*TreeCluster{tc1}, Transporter: v2}
		assert.True(t, next.ShouldRegenerateRoute(prev))
	})

	t.Run("trailer added", func(t *testing.T) {
		prev := &WateringPlan{TreeClusters: []*TreeCluster{tc1}, Transporter: v1}
		next := &WateringPlan{TreeClusters: []*TreeCluster{tc1}, Transporter: v1, Trailer: v2}
		assert.True(t, next.ShouldRegenerateRoute(prev))
	})

	t.Run("cluster order changed", func(t *testing.T) {
		prev := &WateringPlan{TreeClusters: []*TreeCluster{tc1, tc2}, Transporter: v1}
		next := &WateringPlan{TreeClusters: []*TreeCluster{tc2, tc1}, Transporter: v1}
		assert.True(t, next.ShouldRegenerateRoute(prev))
	})

	t.Run("nothing changed", func(t *testing.T) {
		prev := &WateringPlan{TreeClusters: []*TreeCluster{tc1}, Transporter: v1, Trailer: v2}
		next := &WateringPlan{TreeClusters: []*TreeCluster{tc1}, Transporter: v1, Trailer: v2}
		assert.False(t, next.ShouldRegenerateRoute(prev))
	})
}

func TestWateringPlan_IsExpired(t *testing.T) {
	cutoff := time.Now().Add(-24 * time.Hour)

	t.Run("active plan older than cutoff is expired", func(t *testing.T) {
		wp := &WateringPlan{Status: WateringPlanStatusActive, Date: time.Now().Add(-48 * time.Hour)}
		assert.True(t, wp.IsExpired(cutoff))
	})

	t.Run("planned plan older than cutoff is expired", func(t *testing.T) {
		wp := &WateringPlan{Status: WateringPlanStatusPlanned, Date: time.Now().Add(-48 * time.Hour)}
		assert.True(t, wp.IsExpired(cutoff))
	})

	t.Run("recent active plan is not expired", func(t *testing.T) {
		wp := &WateringPlan{Status: WateringPlanStatusActive, Date: time.Now().Add(-1 * time.Hour)}
		assert.False(t, wp.IsExpired(cutoff))
	})

	t.Run("finished plan is not expired regardless of date", func(t *testing.T) {
		wp := &WateringPlan{Status: WateringPlanStatusFinished, Date: time.Now().Add(-48 * time.Hour)}
		assert.False(t, wp.IsExpired(cutoff))
	})

	t.Run("canceled plan is not expired", func(t *testing.T) {
		wp := &WateringPlan{Status: WateringPlanStatusCanceled, Date: time.Now().Add(-48 * time.Hour)}
		assert.False(t, wp.IsExpired(cutoff))
	})
}
